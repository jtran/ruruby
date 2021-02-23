use crate::*;
use std::cell::RefCell;

thread_local!(
    pub static METHODS: RefCell<MethodRepo> = RefCell::new(MethodRepo::new());
);

pub struct MethodRepo {
    table: Vec<MethodInfo>,
    counter: Vec<usize>,
    class_version: u32,
    i_cache: InlineCache,
    m_cache: MethodCache,
}

impl std::ops::Index<MethodId> for MethodRepo {
    type Output = MethodInfo;
    fn index(&self, id: MethodId) -> &MethodInfo {
        &self.table[id.0.get() as usize]
    }
}

impl std::ops::IndexMut<MethodId> for MethodRepo {
    fn index_mut(&mut self, id: MethodId) -> &mut MethodInfo {
        &mut self.table[id.0.get() as usize]
    }
}

impl MethodRepo {
    pub fn new() -> Self {
        Self {
            table: vec![
                MethodInfo::Void, // dummy
                MethodInfo::Void, // default
                MethodInfo::BuiltinFunc {
                    func: enumerator_iterate,
                    name: IdentId::_ENUM_FUNC,
                }, // METHOD_ENUM
            ],
            counter: vec![0, 0, 0],
            class_version: 0,
            i_cache: InlineCache::new(),
            m_cache: MethodCache::new(),
        }
    }

    pub fn add(info: MethodInfo) -> MethodId {
        METHODS.with(|m| {
            let m = &mut m.borrow_mut();
            m.table.push(info);
            m.counter.push(0);
            MethodId::new((m.table.len() - 1) as u32)
        })
    }

    pub fn update(id: MethodId, info: MethodInfo) {
        METHODS.with(|m| {
            m.borrow_mut()[id] = info;
        })
    }

    pub fn update_accessor(id: MethodId, ext: ClassRef, slot: IvarSlot) {
        METHODS.with(|m| {
            let info = &mut m.borrow_mut()[id];
            match info {
                MethodInfo::AttrReader { iv_slot: cache, .. }
                | MethodInfo::AttrWriter { iv_slot: cache, .. } => *cache = Some((ext, slot)),
                _ => unreachable!(),
            }
        })
    }

    pub fn get(id: MethodId) -> MethodInfo {
        METHODS.with(|m| m.borrow()[id].clone())
    }

    pub fn inc_class_version() {
        METHODS.with(|m| m.borrow_mut().class_version += 1)
    }

    pub fn class_version() -> u32 {
        METHODS.with(|m| m.borrow().class_version)
    }

    pub fn add_inline_cache_entry() -> u32 {
        METHODS.with(|m| m.borrow_mut().i_cache.add_entry())
    }

    pub fn get_inline_cache_entry(id: u32) -> InlineCacheEntry {
        METHODS.with(|m| m.borrow().i_cache.get_entry(id))
    }

    pub fn update_inline_cache_entry(id: u32, entry: InlineCacheEntry) {
        METHODS.with(|m| m.borrow_mut().i_cache.update_entry(id, entry))
    }

    /// Search global method cache with receiver class and method name.
    ///
    /// If the method was not found, return None.
    pub fn find_method(rec_class: Module, method_id: IdentId) -> Option<MethodId> {
        METHODS.with(|m| {
            let mut repo = m.borrow_mut();
            let class_version = repo.class_version;
            repo.m_cache.get_method(class_version, rec_class, method_id)
        })
    }

    /// Search global method cache with receiver object and method class_name.
    ///
    /// If the method was not found, return None.
    pub fn find_method_from_receiver(receiver: Value, method_id: IdentId) -> Option<MethodId> {
        let rec_class = receiver.get_class_for_method();
        Self::find_method(rec_class, method_id)
    }

    pub fn mark(alloc: &mut Allocator) {
        let keys: Vec<Module> =
            METHODS.with(|m| m.borrow().m_cache.cache.keys().map(|(v, _)| *v).collect());
        keys.iter().for_each(|m| m.mark(alloc));
    }
}

#[cfg(feature = "perf-method")]
impl MethodRepo {
    pub fn inc_inline_hit() {
        METHODS.with(|m| m.borrow_mut().m_cache.inc_inline_hit());
    }

    pub fn print_method_cache_stats() {
        METHODS.with(|m| m.borrow().m_cache.print_stats());
    }

    pub fn inc_counter(id: MethodId) {
        METHODS.with(|m| m.borrow_mut().counter[id.0.get() as usize] += 1);
    }

    pub fn clear_stats() {
        METHODS.with(|m| {
            m.borrow_mut().counter.iter_mut().for_each(|c| *c = 0);
            m.borrow_mut().m_cache.clear_stats();
        });
    }

    pub fn print_stats() {
        eprintln!("+-------------------------------------------+");
        eprintln!("| Method call stats:                       |");
        eprintln!("+-------------------------------------------+");
        METHODS.with(|m| {
            for (id, count) in m.borrow().counter.iter().enumerate() {
                if *count > 0 {
                    eprintln!(
                        "  MethodId: {:>5}  {:>12}  {:?}",
                        id,
                        count,
                        m.borrow().table[id]
                    );
                }
            }
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MethodId(std::num::NonZeroU32);

impl std::default::Default for MethodId {
    fn default() -> Self {
        Self::new(1)
    }
}

impl MethodId {
    fn new(id: u32) -> Self {
        Self(std::num::NonZeroU32::new(id).unwrap())
    }

    pub fn as_iseq(&self) -> ISeqRef {
        METHODS.with(|m| m.borrow()[*self].as_iseq())
    }
}

impl From<u64> for MethodId {
    fn from(id: u64) -> Self {
        Self::new(id as u32)
    }
}

impl Into<u64> for MethodId {
    fn into(self) -> u64 {
        self.0.get() as u64
    }
}

impl From<u32> for MethodId {
    fn from(id: u32) -> Self {
        Self::new(id)
    }
}

impl Into<usize> for MethodId {
    fn into(self) -> usize {
        self.0.get() as usize
    }
}

pub type BuiltinFunc = fn(vm: &mut VM, self_val: Value, args: &Args) -> VMResult;

pub type MethodTable = FxHashMap<IdentId, MethodId>;

pub static METHOD_ENUM: MethodId = MethodId(unsafe { std::num::NonZeroU32::new_unchecked(2) });

#[derive(Clone)]
pub enum MethodInfo {
    RubyFunc {
        iseq: ISeqRef,
    },
    AttrReader {
        id: IdentId,
        iv_slot: Option<(ClassRef, IvarSlot)>,
    },
    AttrWriter {
        id: IdentId,
        iv_slot: Option<(ClassRef, IvarSlot)>,
    },
    BuiltinFunc {
        name: IdentId,
        func: BuiltinFunc,
    },
    Void,
}

impl GC for MethodInfo {
    fn mark(&self, alloc: &mut Allocator) {
        match self {
            MethodInfo::RubyFunc { iseq } => iseq.class_defined.iter().for_each(|c| c.mark(alloc)),
            _ => return,
        };
    }
}

impl std::fmt::Debug for MethodInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodInfo::RubyFunc { iseq } => write!(f, "RubyFunc {:?}", **iseq),
            MethodInfo::AttrReader { id, iv_slot: cache } => {
                write!(f, "AttrReader {:?} {:?}", id, cache)
            }
            MethodInfo::AttrWriter { id, iv_slot: cache } => {
                write!(f, "AttrWriter {:?} {:?}", id, cache)
            }
            MethodInfo::BuiltinFunc { name, .. } => write!(f, "BuiltinFunc {:?}", name),
            MethodInfo::Void => write!(f, "Void"),
        }
    }
}

impl Default for MethodInfo {
    fn default() -> Self {
        MethodInfo::Void
    }
}

impl MethodInfo {
    pub fn as_iseq(&self) -> ISeqRef {
        if let MethodInfo::RubyFunc { iseq } = self {
            *iseq
        } else {
            unimplemented!("Methodref is illegal.")
        }
    }
}

///---------------------------------------------------------------------------------------------------
///
///  Inline method cache
///
///  This module supports inline method cache which is embedded in the instruction sequence directly.
///
///---------------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct InlineCache {
    table: Vec<InlineCacheEntry>,
    id: u32,
}

#[derive(Debug, Clone)]
pub struct InlineCacheEntry {
    pub version: u32,
    pub entries: Option<(Module, MethodId)>,
}

impl InlineCacheEntry {
    fn new() -> Self {
        InlineCacheEntry {
            version: 0,
            entries: None,
        }
    }
}

impl InlineCache {
    fn new() -> Self {
        InlineCache {
            table: vec![],
            id: 0,
        }
    }
    fn add_entry(&mut self) -> u32 {
        self.id += 1;
        self.table.push(InlineCacheEntry::new());
        self.id - 1
    }

    fn get_entry(&self, id: u32) -> InlineCacheEntry {
        self.table[id as usize].clone()
    }

    fn update_entry(&mut self, id: u32, entry: InlineCacheEntry) {
        self.table[id as usize] = entry;
    }
}

///---------------------------------------------------------------------------------------------------
///
/// Global method cache
///
/// This module supports global method cache.
///
///---------------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct MethodCache {
    cache: FxHashMap<(Module, IdentId), MethodCacheEntry>,
    #[cfg(feature = "perf-method")]
    inline_hit: usize,
    #[cfg(feature = "perf-method")]
    total: usize,
    #[cfg(feature = "perf-method")]
    missed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodCacheEntry {
    pub method: MethodId,
    pub version: u32,
}

impl MethodCache {
    fn new() -> Self {
        MethodCache {
            cache: FxHashMap::default(),
            #[cfg(feature = "perf-method")]
            inline_hit: 0,
            #[cfg(feature = "perf-method")]
            total: 0,
            #[cfg(feature = "perf-method")]
            missed: 0,
        }
    }

    fn add_entry(&mut self, class: Module, id: IdentId, version: u32, method: MethodId) {
        self.cache
            .insert((class, id), MethodCacheEntry { method, version });
    }

    fn get_entry(&self, class: Module, id: IdentId) -> Option<&MethodCacheEntry> {
        self.cache.get(&(class, id))
    }

    /// Get corresponding instance method(MethodId) for the class object `class` and `method`.
    ///
    /// If an entry for `class` and `method` exists in global method cache and the entry is not outdated,
    /// return MethodId of the entry.
    /// If not, search `method` by scanning a class chain.
    /// `class` must be a Class.
    pub fn get_method(
        &mut self,
        class_version: u32,
        rec_class: Module,
        method: IdentId,
    ) -> Option<MethodId> {
        #[cfg(feature = "perf-method")]
        {
            self.total += 1;
        }
        if let Some(MethodCacheEntry { version, method }) = self.get_entry(rec_class, method) {
            if *version == class_version {
                return Some(*method);
            }
        };
        #[cfg(feature = "perf-method")]
        {
            self.missed += 1;
        }
        match rec_class.get_method(method) {
            Some(methodref) => {
                self.add_entry(rec_class, method, class_version, methodref);
                Some(methodref)
            }
            None => None,
        }
    }
}

#[cfg(feature = "perf-method")]
impl MethodCache {
    fn inc_inline_hit(&mut self) {
        self.inline_hit += 1;
    }

    fn clear_stats(&mut self) {
        self.inline_hit = 0;
        self.total = 0;
        self.missed = 0;
    }

    pub fn print_stats(&self) {
        eprintln!("+-------------------------------------------+");
        eprintln!("| Method cache stats:                       |");
        eprintln!("+-------------------------------------------+");
        eprintln!("  hit inline cache : {:>10}", self.inline_hit);
        eprintln!("  hit global cache : {:>10}", self.total - self.missed);
        eprintln!("  missed           : {:>10}", self.missed);
    }
}

//----------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodObjInfo {
    pub name: IdentId,
    pub receiver: Value,
    pub method: MethodId,
}

impl MethodObjInfo {
    pub fn new(name: IdentId, receiver: Value, method: MethodId) -> Self {
        MethodObjInfo {
            name,
            receiver,
            method,
        }
    }
}

impl GC for MethodObjInfo {
    fn mark(&self, alloc: &mut Allocator) {
        self.receiver.mark(alloc);
    }
}
