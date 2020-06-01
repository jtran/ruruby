use std::collections::HashMap;
//#[macro_use]
use crate::*;

/// Heap-allocated objects.
#[derive(Debug, Clone, PartialEq)]
pub struct RValue {
    class: Value,
    var_table: Option<Box<ValueTable>>,
    pub kind: ObjKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjKind {
    Invalid,
    Ordinary,
    Integer(i64),
    Float(f64),
    Class(ClassRef),
    Module(ClassRef),
    String(RString),
    Array(ArrayRef),
    Range(RangeInfo),
    Splat(Value), // internal use only.
    Hash(HashRef),
    Proc(ProcRef),
    Regexp(RegexpRef),
    Method(MethodObjRef),
    Fiber(FiberRef),
    Enumerator(EnumRef),
}

impl GC for RValue {
    fn mark(&self, alloc: &mut Allocator) {
        if alloc.mark(self) {
            return;
        };
        self.class.mark(alloc);
        match &self.var_table {
            Some(table) => table.values().for_each(|v| v.mark(alloc)),
            None => {}
        }
        match self.kind {
            ObjKind::Invalid => panic!("Invalid rvalue. (maybe GC problem) {:?}", &self),
            ObjKind::Class(cref) | ObjKind::Module(cref) => cref.mark(alloc),
            ObjKind::Array(aref) => {
                aref.elements.iter().for_each(|v| v.mark(alloc));
            }
            ObjKind::Hash(href) => href.mark(alloc),
            ObjKind::Range(RangeInfo { start, end, .. }) => {
                start.mark(alloc);
                end.mark(alloc);
            }
            ObjKind::Splat(v) => v.mark(alloc),
            ObjKind::Proc(pref) => pref.context.mark(alloc),
            ObjKind::Method(mref) => mref.mark(alloc),
            ObjKind::Enumerator(eref) => eref.mark(alloc),
            ObjKind::Fiber(fref) => fref.mark(alloc),
            _ => {}
        }
    }
}
impl RValue {
    pub fn free(&mut self) {
        let _ = std::mem::replace(&mut self.var_table, None);
        match self.kind {
            ObjKind::Invalid => panic!("Invalid rvalue. (maybe GC problem) {:?}", self),
            ObjKind::Class(cref) | ObjKind::Module(cref) => cref.free(),
            ObjKind::Array(aref) => aref.free(),
            ObjKind::Hash(href) => href.free(),
            ObjKind::Proc(pref) => pref.free(),
            ObjKind::Method(mref) => mref.free(),
            ObjKind::Enumerator(eref) => eref.free(),
            ObjKind::Fiber(fref) => fref.free(),
            _ => {}
        }
    }
}

impl RValue {
    pub fn id(&self) -> u64 {
        self as *const RValue as u64
    }

    pub fn as_ref(&self) -> ObjectRef {
        Ref::from_ref(self)
    }

    pub fn dup(&self) -> Self {
        RValue {
            class: self.class,
            var_table: self.var_table.clone(),
            kind: match &self.kind {
                ObjKind::Invalid => panic!("Invalid rvalue. (maybe GC problem) {:?}", &self),
                ObjKind::Array(aref) => ObjKind::Array(aref.dup()),
                ObjKind::Class(cref) => ObjKind::Class(cref.dup()),
                ObjKind::Enumerator(eref) => ObjKind::Enumerator(eref.dup()),
                ObjKind::Fiber(_fref) => ObjKind::Ordinary,
                ObjKind::Integer(num) => ObjKind::Integer(*num),
                ObjKind::Float(num) => ObjKind::Float(*num),
                ObjKind::Hash(href) => ObjKind::Hash(href.dup()),
                ObjKind::Method(mref) => ObjKind::Method(mref.dup()),
                ObjKind::Module(cref) => ObjKind::Module(cref.dup()),
                ObjKind::Ordinary => ObjKind::Ordinary,
                ObjKind::Proc(pref) => ObjKind::Proc(pref.dup()),
                ObjKind::Range(info) => ObjKind::Range(info.clone()),
                ObjKind::Regexp(rref) => ObjKind::Regexp(*rref),
                ObjKind::Splat(v) => ObjKind::Splat(*v),
                ObjKind::String(rstr) => ObjKind::String(rstr.clone()),
            },
        }
    }

    pub fn class_name<'a>(&self, globals: &'a Globals) -> &'a str {
        globals.get_ident_name(self.search_class().as_class().name)
    }

    pub fn to_s(&self, globals: &Globals) -> String {
        format! {"#<{}:{:?}>", self.class_name(globals), self}
    }

    pub fn inspect(&self, vm: &mut VM) -> String {
        let mut s = format! {"#<{}:0x{:x}", self.class_name(&vm.globals), self.id()};
        match self.var_table() {
            Some(table) => {
                for (k, v) in table {
                    let inspect = vm.val_to_s(*v);
                    let id = vm.globals.get_ident_name(*k);
                    s = format!("{} {}={}", s, id, inspect);
                }
            }
            None => {}
        }

        format!("{}>", s)
    }

    pub fn new_invalid() -> Self {
        RValue {
            class: Value::nil(),
            kind: ObjKind::Invalid,
            var_table: None,
        }
    }

    pub fn new_bootstrap(classref: ClassRef) -> Self {
        RValue {
            class: Value::nil(), // dummy for boot strapping
            kind: ObjKind::Class(classref),
            var_table: None,
        }
    }

    pub fn new_fixnum(i: i64) -> Self {
        RValue {
            class: Value::nil(),
            var_table: None,
            kind: ObjKind::Integer(i),
        }
    }

    pub fn new_flonum(f: f64) -> Self {
        RValue {
            class: Value::nil(),
            var_table: None,
            kind: ObjKind::Float(f),
        }
    }

    pub fn new_string(globals: &Globals, s: String) -> Self {
        RValue {
            class: globals.builtins.string,
            var_table: None,
            kind: ObjKind::String(RString::Str(s)),
        }
    }

    pub fn new_bytes(globals: &Globals, b: Vec<u8>) -> Self {
        RValue {
            class: globals.builtins.string,
            var_table: None,
            kind: ObjKind::String(RString::Bytes(b)),
        }
    }

    pub fn new_ordinary(class: Value) -> Self {
        RValue {
            class,
            var_table: None,
            kind: ObjKind::Ordinary,
        }
    }

    pub fn new_class(globals: &Globals, classref: ClassRef) -> Self {
        RValue {
            class: globals.builtins.class,
            var_table: None,
            kind: ObjKind::Class(classref),
        }
    }

    pub fn new_module(globals: &Globals, classref: ClassRef) -> Self {
        RValue {
            class: globals.builtins.module,
            var_table: None,
            kind: ObjKind::Module(classref),
        }
    }

    pub fn new_array(globals: &Globals, arrayref: ArrayRef) -> Self {
        RValue {
            class: globals.builtins.array,
            var_table: None,
            kind: ObjKind::Array(arrayref),
        }
    }

    pub fn new_range(globals: &Globals, range: RangeInfo) -> Self {
        RValue {
            class: globals.builtins.range,
            var_table: None,
            kind: ObjKind::Range(range),
        }
    }

    pub fn new_splat(globals: &Globals, val: Value) -> Self {
        RValue {
            class: globals.builtins.array,
            var_table: None,
            kind: ObjKind::Splat(val),
        }
    }

    pub fn new_hash(globals: &Globals, hashref: HashRef) -> Self {
        RValue {
            class: globals.builtins.hash,
            var_table: None,
            kind: ObjKind::Hash(hashref),
        }
    }

    pub fn new_regexp(globals: &Globals, regexpref: RegexpRef) -> Self {
        RValue {
            class: globals.builtins.regexp,
            var_table: None,
            kind: ObjKind::Regexp(regexpref),
        }
    }

    pub fn new_proc(globals: &Globals, procref: ProcRef) -> Self {
        RValue {
            class: globals.builtins.procobj,
            var_table: None,
            kind: ObjKind::Proc(procref),
        }
    }

    pub fn new_method(globals: &Globals, methodref: MethodObjRef) -> Self {
        RValue {
            class: globals.builtins.method,
            var_table: None,
            kind: ObjKind::Method(methodref),
        }
    }

    pub fn new_fiber(
        globals: &Globals,
        vm: VMRef,
        context: ContextRef,
        rec: std::sync::mpsc::Receiver<VMResult>,
        tx: std::sync::mpsc::SyncSender<usize>,
    ) -> Self {
        let fiber = FiberInfo::new(vm, context, rec, tx);
        RValue {
            class: globals.builtins.fiber,
            var_table: None,
            kind: ObjKind::Fiber(FiberRef::new(fiber)),
        }
    }

    pub fn new_enumerator(
        globals: &Globals,
        method: IdentId,
        receiver: Value,
        mut args: Args,
    ) -> Self {
        args.block = Some(MethodRef::from(0));
        let enum_info = EnumRef::from(method, receiver, args);
        RValue {
            class: globals.builtins.enumerator,
            var_table: None,
            kind: ObjKind::Enumerator(enum_info),
        }
    }
}

pub type ObjectRef = Ref<RValue>;

impl RValue {
    /// Pack `self` into `Value`(64-bit data representation).
    /// This method consumes `self` and allocates it on the heap, returning `Value`,
    /// a wrapped raw pointer.  
    pub fn pack(self) -> Value {
        Value::from(ALLOC.lock().unwrap().alloc(self) as u64)
    }

    /// Return a class of the object. If the objetct has a sigleton class, return the singleton class.
    pub fn class(&self) -> Value {
        self.class
    }

    /// Return a "real" class of the object.
    pub fn search_class(&self) -> Value {
        let mut class = self.class;
        loop {
            if class.as_class().is_singleton {
                class = class.as_object().class;
            } else {
                return class;
            }
        }
    }

    /// Set a class of the object.
    pub fn set_class(&mut self, class: Value) {
        self.class = class;
    }

    pub fn get_var(&self, id: IdentId) -> Option<Value> {
        match &self.var_table {
            Some(table) => table.get(&id).cloned(),
            None => None,
        }
    }

    pub fn get_mut_var(&mut self, id: IdentId) -> Option<&mut Value> {
        match &mut self.var_table {
            Some(table) => table.get_mut(&id),
            None => None,
        }
    }

    pub fn set_var(&mut self, id: IdentId, val: Value) {
        match &mut self.var_table {
            Some(table) => table.insert(id, val),
            None => {
                let mut table = HashMap::new();
                let v = table.insert(id, val);
                self.var_table = Some(Box::new(table));
                v
            }
        };
    }

    pub fn var_table(&self) -> Option<&ValueTable> {
        match &self.var_table {
            Some(table) => Some(table),
            None => None,
        }
    }

    pub fn var_table_mut(&mut self) -> &mut ValueTable {
        if self.var_table.is_none() {
            self.var_table = Some(Box::new(HashMap::new()));
        }
        self.var_table.as_deref_mut().unwrap()
    }

    pub fn get_instance_method(&self, id: IdentId) -> Option<MethodRef> {
        self.search_class()
            .as_class()
            .method_table
            .get(&id)
            .cloned()
    }
}
