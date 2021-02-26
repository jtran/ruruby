use crate::coroutine::*;
use crate::*;
use std::borrow::Cow;

/// Heap-allocated objects.
#[derive(Debug, PartialEq)]
pub struct RValue {
    class: Module,
    ivars: Option<IvarTable>,
    ext: ClassRef,
    pub kind: ObjKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IvarTable(Box<Vec<Option<Value>>>);

impl std::ops::Deref for IvarTable {
    type Target = Vec<Option<Value>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IvarTable {
    fn new(ext: ClassRef) -> Self {
        let len = ext.ivar_len();
        Self(Box::new(vec![None; len]))
    }

    /// Get value in `slot` of `self`.
    /// If `slot` is not available, return `None`.
    pub fn get(&self, slot: IvarSlot) -> Option<Value> {
        self.0.get(slot.into_usize()).cloned().flatten()
    }

    /// Get value in `slot` of `self`.
    /// If `slot` is not available, return `None`.
    fn get_mut(&mut self, slot: IvarSlot) -> Option<&mut Option<Value>> {
        self.0.get_mut(slot.into_usize())
    }
}

#[derive(Debug, PartialEq)]
pub enum ObjKind {
    Invalid,
    Ordinary,
    Integer(i64),
    Float(f64),
    Complex { r: Value, i: Value },
    Module(ClassInfo),
    String(Box<RString>),
    Array(ArrayInfo),
    Range(RangeInfo),
    Splat(Value), // internal use only.
    Hash(Box<HashInfo>),
    Proc(ProcInfo),
    Regexp(RegexpInfo),
    Method(MethodObjInfo),
    Fiber(Box<FiberContext>),
    Enumerator(Box<FiberContext>),
    Time(TimeInfo),
    Exception(RubyError),
}

impl GC for RValue {
    fn mark(&self, alloc: &mut Allocator) {
        self.class.mark(alloc);
        if let Some(info) = &self.ivars {
            info.0.iter().for_each(|v| {
                if let Some(v) = v {
                    v.mark(alloc)
                }
            })
        };
        match &self.kind {
            ObjKind::Invalid => panic!(
                "Invalid rvalue. (maybe GC problem) {:?} {:#?}",
                self as *const RValue, self
            ),
            ObjKind::Ordinary => {}
            ObjKind::Complex { r, i } => {
                r.mark(alloc);
                i.mark(alloc);
            }
            ObjKind::Module(cref) => cref.mark(alloc),
            ObjKind::Array(aref) => aref.mark(alloc),
            ObjKind::Hash(href) => href.mark(alloc),
            ObjKind::Range(RangeInfo { start, end, .. }) => {
                start.mark(alloc);
                end.mark(alloc);
            }
            ObjKind::Splat(v) => v.mark(alloc),
            ObjKind::Proc(pref) => pref.context.mark(alloc),
            ObjKind::Method(mref) => mref.mark(alloc),
            ObjKind::Enumerator(fref) | ObjKind::Fiber(fref) => fref.mark(alloc),
            ObjKind::Exception(err) => match &err.kind {
                RubyErrorKind::Value(val) => val.mark(alloc),
                RubyErrorKind::BlockReturn(val) => val.mark(alloc),
                RubyErrorKind::MethodReturn(val) => val.mark(alloc),
                _ => {}
            },
            _ => {}
        }
    }
}

impl RValue {
    pub fn free(&mut self) -> bool {
        if self.kind == ObjKind::Invalid {
            return false;
        };
        self.kind = ObjKind::Invalid;
        self.ivars = None;
        true
    }
}

impl RValue {
    pub fn id(&self) -> u64 {
        self as *const RValue as u64
    }

    pub fn as_ref(&self) -> ObjectRef {
        Ref::from_ref(self)
    }

    pub fn ext(&self) -> ClassRef {
        self.ext
    }

    pub fn dup(&self) -> Self {
        // TODO: Is it correct?
        RValue {
            class: self.class,
            ext: self.ext,
            ivars: self.ivars.clone(),
            kind: match &self.kind {
                ObjKind::Invalid => panic!("Invalid rvalue. (maybe GC problem) {:?}", &self),
                ObjKind::Complex { r, i } => ObjKind::Complex {
                    r: r.dup(),
                    i: i.dup(),
                },
                ObjKind::Array(aref) => ObjKind::Array(aref.clone()),
                ObjKind::Module(cinfo) => ObjKind::Module(cinfo.clone()),
                ObjKind::Enumerator(_eref) => unreachable!(),
                ObjKind::Fiber(_fref) => unreachable!(),
                ObjKind::Integer(num) => ObjKind::Integer(*num),
                ObjKind::Float(num) => ObjKind::Float(*num),
                ObjKind::Hash(hinfo) => ObjKind::Hash(hinfo.clone()),
                ObjKind::Method(hinfo) => ObjKind::Method(hinfo.clone()),
                ObjKind::Ordinary => ObjKind::Ordinary,
                ObjKind::Proc(pref) => ObjKind::Proc(pref.clone()),
                ObjKind::Range(info) => ObjKind::Range(info.clone()),
                ObjKind::Regexp(rref) => ObjKind::Regexp(rref.clone()),
                ObjKind::Splat(v) => ObjKind::Splat(*v),
                ObjKind::String(rstr) => ObjKind::String(rstr.clone()),
                ObjKind::Time(time) => ObjKind::Time(time.clone()),
                ObjKind::Exception(err) => ObjKind::Exception(err.clone()),
            },
        }
    }

    pub fn class_name(&self) -> String {
        self.search_class().name()
    }

    pub fn ivars(&self) -> &Option<IvarTable> {
        &self.ivars
    }

    pub fn ivar_seek(&self, slot: IvarSlot) -> Option<Value> {
        match &self.ivars {
            Some(info) => info.get(slot),
            None => None,
        }
    }

    pub fn ivar_get(&mut self, slot: IvarSlot) -> Option<Value> {
        let ext = self.ext();
        match &mut self.ivars {
            Some(info) => info.get_mut(slot).map(|v| *v).flatten(),
            None => {
                let table = IvarTable::new(ext);
                self.ivars = Some(table);
                None
            }
        }
    }

    pub fn ivar_set(&mut self, slot: IvarSlot, val: Option<Value>) {
        let ext = self.ext();
        match &mut self.ivars {
            Some(table) => match table.get_mut(slot) {
                Some(v) => {
                    *v = val;
                }
                None => {
                    table.0.resize(ext.ivar_len(), None);
                    table.0[slot.into_usize()] = val;
                }
            },
            None => {
                let mut table = IvarTable::new(ext);
                table.0[slot.into_usize()] = val;
                self.ivars = Some(table);
            }
        }
    }

    pub fn to_s(&self) -> String {
        format! {"#<{}:0x{:016x}>", self.class_name(), self.id()}
    }

    /// Create new RValue with `class` and `kind`.
    pub fn new(class: Module, kind: ObjKind) -> Self {
        let ext = class.ext();
        Self {
            class,
            ext,
            ivars: Some(IvarTable::new(ext)),
            kind,
        }
    }

    pub fn new_invalid() -> Self {
        Self {
            class: Module::default(),
            ext: unsafe { ClassRef::new_unchecked() },
            ivars: None,
            kind: ObjKind::Invalid,
        }
    }

    pub fn new_bootstrap(cinfo: ClassInfo) -> Self {
        Self {
            class: Module::default(),
            ext: unsafe { ClassRef::new_unchecked() },
            ivars: None,
            kind: ObjKind::Module(cinfo),
        }
    }

    pub fn new_integer(i: i64) -> Self {
        RValue::new(BuiltinClass::integer(), ObjKind::Integer(i))
    }

    pub fn new_float(f: f64) -> Self {
        RValue::new(BuiltinClass::float(), ObjKind::Float(f))
    }

    pub fn new_complex(r: Value, i: Value) -> Self {
        let class = BuiltinClass::complex();
        RValue::new(class, ObjKind::Complex { r, i })
    }

    pub fn new_string_from_rstring(rs: RString) -> Self {
        RValue::new(BuiltinClass::string(), ObjKind::String(Box::new(rs)))
    }

    pub fn new_string_derive_from_rstring(rs: RString, class: Module) -> Self {
        RValue::new(class, ObjKind::String(Box::new(rs)))
    }

    pub fn new_string<'a>(s: impl Into<Cow<'a, str>>) -> Self {
        RValue::new_string_from_rstring(RString::from(s))
    }

    pub fn new_string_derive<'a>(s: impl Into<Cow<'a, str>>, class: Module) -> Self {
        RValue::new_string_derive_from_rstring(RString::from(s), class)
    }

    pub fn new_bytes(b: Vec<u8>) -> Self {
        RValue::new_string_from_rstring(RString::Bytes(b))
    }

    pub fn new_ordinary(class: Module) -> Self {
        RValue::new(class, ObjKind::Ordinary)
    }

    pub fn new_class(cinfo: ClassInfo) -> Self {
        RValue::new(BuiltinClass::class(), ObjKind::Module(cinfo))
    }

    pub fn new_module(cinfo: ClassInfo) -> Self {
        RValue::new(BuiltinClass::module(), ObjKind::Module(cinfo))
    }

    pub fn new_array(array_info: ArrayInfo) -> Self {
        RValue::new(BuiltinClass::array(), ObjKind::Array(array_info))
    }

    pub fn new_array_derive(array_info: ArrayInfo, class: Module) -> Self {
        RValue::new(class, ObjKind::Array(array_info))
    }

    pub fn new_range(range: RangeInfo) -> Self {
        RValue::new(BuiltinClass::range(), ObjKind::Range(range))
    }

    pub fn new_splat(val: Value) -> Self {
        RValue::new(BuiltinClass::array(), ObjKind::Splat(val))
    }

    pub fn new_hash(hash: HashInfo) -> Self {
        RValue::new(BuiltinClass::hash(), ObjKind::Hash(Box::new(hash)))
    }

    pub fn new_regexp(regexp: RegexpInfo) -> Self {
        RValue::new(BuiltinClass::regexp(), ObjKind::Regexp(regexp))
    }

    pub fn new_proc(proc_info: ProcInfo) -> Self {
        RValue::new(BuiltinClass::procobj(), ObjKind::Proc(proc_info))
    }

    pub fn new_method(method_info: MethodObjInfo) -> Self {
        RValue::new(BuiltinClass::method(), ObjKind::Method(method_info))
    }

    pub fn new_fiber(vm: VM, context: ContextRef) -> Self {
        let fiber = FiberContext::new_fiber(vm, context);
        RValue::new(BuiltinClass::fiber(), ObjKind::Fiber(fiber))
    }

    pub fn new_enumerator(fiber: Box<FiberContext>) -> Self {
        RValue::new(BuiltinClass::enumerator(), ObjKind::Enumerator(fiber))
    }

    pub fn new_time(time_class: Module, time: TimeInfo) -> Self {
        RValue::new(time_class, ObjKind::Time(time))
    }

    pub fn new_exception(exception_class: Module, err: RubyError) -> Self {
        RValue::new(exception_class, ObjKind::Exception(err))
    }
}

pub type ObjectRef = Ref<RValue>;

impl RValue {
    /// Pack `self` into `Value`(64-bit data representation).
    ///
    /// This method consumes `self` and allocates it on the heap, returning `Value`,
    /// a wrapped raw pointer.  
    pub fn pack(self) -> Value {
        let ptr = ALLOC.with(|alloc| {
            alloc.borrow_mut().alloc(self)
            //assert!((ptr as u64) & 0b111 == 0);
        });
        Value::from_ptr(ptr)
    }

    /// Return a class of the object.
    ///
    /// If the objetct has a sigleton class, return the singleton class.
    pub fn class(&self) -> Module {
        self.class
    }

    /// Return a "real" class of the object.
    pub fn search_class(&self) -> Module {
        let mut class = self.class;
        while class.is_singleton() {
            class = class.superclass().unwrap();
        }
        class
    }

    /// Set a class of the object.
    pub unsafe fn set_class(&mut self, class: Module) {
        self.class = class;
    }

    /// Change a class and ext of the object.
    pub unsafe fn change_class(&mut self, class: Module) {
        self.class = class;
        self.ext = class.ext();
    }

    pub fn get_singleton(&mut self, org_val: Value) -> Module {
        let singleton = match &self.kind {
            ObjKind::Module(cinfo) => {
                let superclass = match cinfo.superclass() {
                    None => None,
                    Some(superclass) => Some(superclass.get_singleton_class()),
                };
                Module::singleton_class_from(superclass, org_val)
            }
            ObjKind::Invalid => {
                panic!("Invalid rvalue. (maybe GC problem) {:?}", *self)
            }
            _ => Module::singleton_class_from(self.class(), org_val),
        };
        self.class = singleton;
        singleton
    }
}
