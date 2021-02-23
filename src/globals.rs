use crate::*;
use fancy_regex::Regex;
use std::path::PathBuf;
use std::rc::Rc;
mod constants;
use constants::*;

#[derive(Debug, Clone)]
pub struct Globals {
    // Global info
    pub const_values: ConstantValues,
    global_var: ValueTable,
    const_cache: ConstCache,
    pub case_dispatch: CaseDispatchMap,

    main_fiber: Option<VMRef>,
    pub instant: std::time::Instant,
    pub const_version: u32,
    pub main_object: Value,
    pub regexp_cache: FxHashMap<String, Rc<Regex>>,
    source_files: Vec<PathBuf>,
}

pub type GlobalsRef = Ref<Globals>;

impl GC for Globals {
    fn mark(&self, alloc: &mut Allocator) {
        self.const_values.mark(alloc);
        self.main_object.mark(alloc);
        self.global_var.values().for_each(|v| v.mark(alloc));
        MethodRepo::mark(alloc);
        for t in &self.case_dispatch.table {
            t.keys().for_each(|k| k.mark(alloc));
        }
        if let Some(vm) = self.main_fiber {
            vm.mark(alloc);
        }
    }
}

impl GlobalsRef {
    pub fn new_globals() -> Self {
        Ref::new(Globals::new())
    }

    pub fn create_main_fiber(&mut self) -> VMRef {
        let vm = VMRef::new(VM::new(self.to_owned()));
        self.main_fiber = Some(vm);
        vm
    }
}

impl Globals {
    fn new() -> Self {
        use builtin::*;
        let object = BuiltinClass::object();
        let main_object = Value::ordinary_object(object);
        let mut globals = Globals {
            const_values: ConstantValues::new(),
            global_var: FxHashMap::default(),
            const_cache: ConstCache::new(),
            main_fiber: None,
            instant: std::time::Instant::now(),
            const_version: 0,
            main_object,
            case_dispatch: CaseDispatchMap::new(),
            regexp_cache: FxHashMap::default(),
            source_files: vec![],
        };

        BUILTINS.with(|_| {});
        let stdout = BuiltinClass::get_toplevel_constant("STDOUT").unwrap();
        globals.set_global_var_by_str("$>", stdout);
        globals.set_global_var_by_str("$stdout", stdout);

        let mut env_map = HashInfo::new(FxHashMap::default());
        std::env::vars()
            .for_each(|(var, val)| env_map.insert(Value::string(var), Value::string(val)));
        let env = Value::hash_from(env_map);
        globals.set_toplevel_constant("ENV", env);
        globals
    }

    pub fn gc(&self) {
        ALLOC.with(|m| m.borrow_mut().gc(self));
    }

    pub fn add_source_file(&mut self, file_path: &PathBuf) -> Option<usize> {
        if self.source_files.contains(file_path) {
            None
        } else {
            let i = self.source_files.len();
            self.source_files.push(file_path.to_owned());
            Some(i)
        }
    }

    #[cfg(feature = "gc-debug")]
    pub fn print_mark(&self) {
        ALLOC.with(|m| m.borrow_mut().print_mark());
    }

    #[cfg(feature = "perf-method")]
    pub fn clear_const_cache(&mut self) {
        self.const_cache.clear();
    }
}

impl Globals {
    pub fn get_global_var(&self, id: IdentId) -> Option<Value> {
        self.global_var.get(&id).cloned()
    }

    pub fn set_global_var(&mut self, id: IdentId, val: Value) {
        self.global_var.insert(id, val);
    }

    pub fn set_global_var_by_str(&mut self, name: &str, val: Value) {
        let id = IdentId::get_id(name);
        self.global_var.insert(id, val);
    }

    /// Bind `class_object` to the constant `class_name` of the root object.
    pub fn set_const(&mut self, mut class_obj: Module, class_name: IdentId, val: impl Into<Value>) {
        let val = val.into();
        class_obj.set_const(class_name, val);
        self.const_version += 1;
    }

    /// Search inline constant cache for `slot`.
    ///
    /// Return None if not found.
    pub fn find_const_cache(&mut self, slot: u32) -> Option<Value> {
        let const_version = self.const_version;
        #[cfg(feature = "perf-method")]
        {
            self.const_cache.total += 1;
        }
        match &self.const_cache.get_entry(slot) {
            ConstCacheEntry {
                version,
                val: Some(val),
            } if *version == const_version => Some(*val),
            _ => {
                #[cfg(feature = "perf-method")]
                {
                    self.const_cache.missed += 1;
                }
                None
            }
        }
    }

    /// Bind `object` to the constant `name` of the root object.
    pub fn set_toplevel_constant(&mut self, name: &str, object: impl Into<Value>) {
        BuiltinClass::object().set_const_by_str(name, object.into());
        self.const_version += 1;
    }
}

impl Globals {
    pub fn add_const_cache_entry(&mut self) -> u32 {
        self.const_cache.add_entry()
    }

    pub fn set_const_cache(&mut self, id: u32, val: Value) {
        let version = self.const_version;
        self.const_cache.set(id, version, val)
    }
}

#[cfg(feature = "perf-method")]
impl Globals {
    pub fn print_constant_cache_stats(&self) {
        self.const_cache.print_stats()
    }
}

///
/// Contant value
///
/// A table which holds constant values.
///
#[derive(Debug, Clone)]
pub struct ConstantValues {
    table: Vec<Value>,
}

impl ConstantValues {
    pub fn new() -> Self {
        Self { table: vec![] }
    }

    pub fn insert(&mut self, val: Value) -> usize {
        let id = self.table.len();
        self.table.push(val);
        id
    }

    pub fn get(&self, id: usize) -> Value {
        self.table[id].dup()
    }

    #[cfg(feature = "emit-iseq")]
    pub fn dump(&self) {
        for (i, val) in self.table.iter().enumerate() {
            eprintln!("{}:{:?}", i, val);
        }
    }
}

impl GC for ConstantValues {
    fn mark(&self, alloc: &mut Allocator) {
        self.table.iter().for_each(|v| v.mark(alloc));
    }
}

///
/// Case dispatch map.
///
/// This module supports optimization for case-when syntax when all of the when-conditions were integer literals.
///
#[derive(Debug, Clone)]
pub struct CaseDispatchMap {
    table: Vec<FxHashMap<Value, i32>>,
    id: u32,
}

impl CaseDispatchMap {
    fn new() -> Self {
        CaseDispatchMap {
            table: vec![],
            id: 0,
        }
    }

    pub fn new_entry(&mut self) -> u32 {
        self.id += 1;
        self.table.push(FxHashMap::default());
        self.id - 1
    }

    pub fn get_entry(&self, id: u32) -> &FxHashMap<Value, i32> {
        &self.table[id as usize]
    }

    pub fn get_mut_entry(&mut self, id: u32) -> &mut FxHashMap<Value, i32> {
        &mut self.table[id as usize]
    }
}
