use crate::coroutine::*;
use crate::*;
use smallvec::SmallVec;
use std::borrow::Cow;

/// Heap-allocated objects.
#[derive(Debug, PartialEq)]
pub struct RValue {
    class: Module,
    ivars: IvarTable,
    pub kind: ObjKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IvarInfo {
    vec: SmallVec<[Option<Value>; 8]>,
    ext: ClassRef,
}

impl IvarInfo {
    pub fn new(len: usize, ext: ClassRef) -> Self {
        Self {
            vec: smallvec![None;len],
            ext,
        }
    }

    pub fn ext(&self) -> ClassRef {
        self.ext
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn get(&self, slot: IvarSlot) -> Option<Option<Value>> {
        let slot = slot.into_usize();
        if slot >= self.len() {
            None
        } else {
            Some(self.vec[slot])
        }
    }

    pub fn get_mut(&mut self, slot: IvarSlot) -> Option<&mut Option<Value>> {
        let slot = slot.into_usize();
        if slot >= self.len() {
            None
        } else {
            Some(&mut self.vec[slot])
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IvarTable(Option<Box<IvarInfo>>);

impl IvarTable {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn new_with_ext(ext: ClassRef) -> Self {
        Self(Some(Box::new(IvarInfo::new(ext.ivar_len(), ext))))
    }

    pub fn ext(&self) -> Option<ClassRef> {
        self.0.as_ref().map(|info| info.ext())
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |v| v.len())
    }

    pub fn get(&self, slot: IvarSlot) -> Option<Option<Value>> {
        self.0.as_ref().and_then(|v| v.get(slot))
    }

    pub fn access(&self, slot: IvarSlot) -> Value {
        self.get(slot).flatten().unwrap_or_default()
    }

    pub fn set(&mut self, slot: IvarSlot, val: Option<Value>, ext: ClassRef) {
        match &mut self.0 {
            Some(info) => match info.get_mut(slot) {
                Some(v) => *v = val,
                None => {
                    info.vec.resize(ext.ivar_len(), None);
                    info.vec[slot.into_usize()] = val;
                }
            },
            None => {
                let mut info = IvarInfo::new(ext.ivar_len(), ext);
                info.vec[slot.into_usize()] = val;
                self.0 = Some(Box::new(info));
            }
        }
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
    String(RString),
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
        if let Some(info) = &self.ivars.0 {
            info.vec.iter().for_each(|v| {
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
        self.ivars = IvarTable::new();
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

    pub fn dup(&self) -> Self {
        RValue {
            class: self.class,
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

    pub fn get_ext(&mut self, org_val: Value) -> ClassRef {
        match self.ivars.ext() {
            Some(ext) => ext,
            None => {
                //eprintln!("init");
                let ext = org_val.get_class().ext();
                self.ivars = IvarTable::new_with_ext(ext);
                ext
            }
        }
    }

    pub fn to_s(&self) -> String {
        format! {"#<{}:0x{:016x}>", self.class_name(), self.id()}
    }

    pub fn new(class: Module, kind: ObjKind) -> Self {
        RValue {
            class,
            kind,
            ivars: IvarTable::new(),
        }
    }

    pub fn new_invalid() -> Self {
        RValue {
            class: Module::default(),
            kind: ObjKind::Invalid,
            ivars: IvarTable::new(),
        }
    }

    pub fn new_bootstrap(cinfo: ClassInfo) -> Self {
        RValue {
            class: Module::default(), // dummy for boot strapping
            kind: ObjKind::Module(cinfo),
            ivars: IvarTable::new(),
        }
    }

    pub fn new_integer(i: i64) -> Self {
        RValue {
            class: BuiltinClass::integer(),
            ivars: IvarTable::new(),
            kind: ObjKind::Integer(i),
        }
    }

    pub fn new_float(f: f64) -> Self {
        RValue {
            class: BuiltinClass::float(),
            ivars: IvarTable::new(),
            kind: ObjKind::Float(f),
        }
    }

    pub fn new_complex(r: Value, i: Value) -> Self {
        let class = BuiltinClass::complex();
        RValue {
            class,
            ivars: IvarTable::new(),
            kind: ObjKind::Complex { r, i },
        }
    }

    pub fn new_string_from_rstring(rs: RString) -> Self {
        RValue {
            class: BuiltinClass::string(),
            ivars: IvarTable::new(),
            kind: ObjKind::String(rs),
        }
    }

    pub fn new_string<'a>(s: impl Into<Cow<'a, str>>) -> Self {
        RValue::new_string_from_rstring(RString::from(s))
    }

    pub fn new_bytes(b: Vec<u8>) -> Self {
        RValue::new_string_from_rstring(RString::Bytes(b))
    }

    pub fn new_ordinary(class: Module) -> Self {
        RValue {
            class,
            ivars: IvarTable::new(),
            kind: ObjKind::Ordinary,
        }
    }

    pub fn new_class(cinfo: ClassInfo) -> Self {
        RValue {
            class: BuiltinClass::class(),
            ivars: IvarTable::new(),
            kind: ObjKind::Module(cinfo),
        }
    }

    pub fn new_module(cinfo: ClassInfo) -> Self {
        RValue {
            class: BuiltinClass::module(),
            ivars: IvarTable::new(),
            kind: ObjKind::Module(cinfo),
        }
    }

    pub fn new_array(array_info: ArrayInfo) -> Self {
        RValue {
            class: BuiltinClass::array(),
            ivars: IvarTable::new(),
            kind: ObjKind::Array(array_info),
        }
    }

    pub fn new_array_with_class(array_info: ArrayInfo, class: Module) -> Self {
        RValue {
            class,
            ivars: IvarTable::new(),
            kind: ObjKind::Array(array_info),
        }
    }

    pub fn new_range(range: RangeInfo) -> Self {
        RValue {
            class: BuiltinClass::range(),
            ivars: IvarTable::new(),
            kind: ObjKind::Range(range),
        }
    }

    pub fn new_splat(val: Value) -> Self {
        RValue {
            class: BuiltinClass::array(),
            ivars: IvarTable::new(),
            kind: ObjKind::Splat(val),
        }
    }

    pub fn new_hash(hash: HashInfo) -> Self {
        RValue {
            class: BuiltinClass::hash(),
            ivars: IvarTable::new(),
            kind: ObjKind::Hash(Box::new(hash)),
        }
    }

    pub fn new_regexp(regexp: RegexpInfo) -> Self {
        RValue {
            class: BuiltinClass::regexp(),
            ivars: IvarTable::new(),
            kind: ObjKind::Regexp(regexp),
        }
    }

    pub fn new_proc(proc_info: ProcInfo) -> Self {
        RValue {
            class: BuiltinClass::procobj(),
            ivars: IvarTable::new(),
            kind: ObjKind::Proc(proc_info),
        }
    }

    pub fn new_method(method_info: MethodObjInfo) -> Self {
        RValue {
            class: BuiltinClass::method(),
            ivars: IvarTable::new(),
            kind: ObjKind::Method(method_info),
        }
    }

    pub fn new_fiber(vm: VM, context: ContextRef) -> Self {
        let fiber = FiberContext::new_fiber(vm, context);
        RValue {
            class: BuiltinClass::fiber(),
            ivars: IvarTable::new(),
            kind: ObjKind::Fiber(fiber),
        }
    }

    pub fn new_enumerator(fiber: Box<FiberContext>) -> Self {
        RValue {
            class: BuiltinClass::enumerator(),
            ivars: IvarTable::new(),
            kind: ObjKind::Enumerator(fiber),
        }
    }

    pub fn new_time(time_class: Module, time: TimeInfo) -> Self {
        RValue {
            class: time_class,
            ivars: IvarTable::new(),
            kind: ObjKind::Time(time),
        }
    }

    pub fn new_exception(exception_class: Module, err: RubyError) -> Self {
        RValue {
            class: exception_class,
            ivars: IvarTable::new(),
            kind: ObjKind::Exception(err),
        }
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
    pub fn set_class(&mut self, class: Module) {
        self.class = class;
    }

    pub fn ivars(&mut self) -> &mut IvarTable {
        &mut self.ivars
    }
}
