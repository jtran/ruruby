use crate::*;
#[cfg(all(unix, target_arch = "x86_64"))]
mod asm_x64;
mod stack;
#[cfg(all(unix, target_arch = "x86_64"))]
use asm_x64 as asm;
use stack::*;

#[derive(PartialEq, Eq, Debug)]
pub enum FiberState {
    Created,
    Running,
    Dead,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FiberKind {
    Fiber(ContextRef),
    Enum(Box<EnumInfo>),
}

impl GC for FiberKind {
    fn mark(&self, alloc: &mut Allocator) {
        match self {
            FiberKind::Fiber(context) => context.mark(alloc),
            FiberKind::Enum(info) => info.mark(alloc),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnumInfo {
    pub receiver: Value,
    pub method: IdentId,
    pub args: Args,
}

impl GC for EnumInfo {
    fn mark(&self, alloc: &mut Allocator) {
        self.receiver.mark(alloc);
        self.args.mark(alloc);
    }
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct FiberContext {
    rsp: u64,
    main_rsp: u64,
    stack: Stack,
    pub state: FiberState,
    pub vm: VMRef,
    pub kind: FiberKind,
}

impl Drop for FiberContext {
    fn drop(&mut self) {
        //eprintln!("dropped!");
        self.vm.free();
        self.stack.deallocate();
    }
}

impl GC for FiberContext {
    fn mark(&self, alloc: &mut Allocator) {
        if self.state == FiberState::Dead {
            return;
        }
        self.vm.mark(alloc);
        self.kind.mark(alloc);
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FiberHandle(*mut FiberContext);

impl FiberHandle {
    pub fn vm(&self) -> VMRef {
        unsafe { (*self.0).vm }
    }

    pub fn kind(&self) -> &FiberKind {
        unsafe { &(*self.0).kind }
    }

    /// Yield args to parent fiber. (execute Fiber.yield)
    pub fn fiber_yield(vm: &mut VM, args: &Args) -> VMResult {
        let val = match args.len() {
            0 => Value::nil(),
            1 => args[0],
            _ => Value::array_from(args.to_vec()),
        };
        match vm.handle {
            None => Err(RubyError::fiber("Can not yield from main fiber.")),
            Some(handle) => {
                #[cfg(feature = "perf")]
                vm.globals.perf.get_perf(Perf::INVALID);
                #[cfg(any(feature = "trace", feature = "trace-func"))]
                println!("<=== yield Ok({:?})", val);
                let send_val = Box::into_raw(Box::new(Ok(val)));
                let val = asm::yield_context(handle.0, send_val);
                Ok(Value::from(val))
            }
        }
    }
}

impl FiberContext {
    //            stack end
    //     +---------------------+
    // -8  |  *mut FiberContext  |
    //     +---------------------+
    // -16 |        guard        |
    //     +---------------------+
    // -24 |        skip         |
    //     +---------------------+
    // -32 |          f          |
    //     +---------------------+
    //     |                     |
    //     |     callee-save     |
    //     |      registers      |
    // -80 |                     | <-sp
    //     +---------------------+
    pub fn spawn(vm: VMRef, kind: FiberKind) -> Box<Self> {
        let mut fiber = Box::new(FiberContext::new(vm, kind));
        let ptr = &*fiber as *const _;
        fiber.rsp = fiber.stack.init(ptr, Self::new_context);
        fiber
    }
}

impl FiberContext {
    fn new(vm: VMRef, kind: FiberKind) -> Self {
        FiberContext {
            rsp: 0,
            main_rsp: 0,
            stack: Stack::allocate(),
            state: FiberState::Created,
            vm,
            kind,
        }
    }

    pub fn new_fiber(vm: VM, context: ContextRef) -> Box<Self> {
        let vmref = VMRef::new(vm);
        Self::spawn(vmref, FiberKind::Fiber(context))
    }

    pub fn new_enumerator(vm: VM, info: EnumInfo) -> Box<Self> {
        let vmref = VMRef::new(vm);
        Self::spawn(vmref, FiberKind::Enum(Box::new(info)))
    }

    extern "C" fn new_context(handle: FiberHandle, _val: Value) -> *mut VMResult {
        let mut fiber_vm = handle.vm();
        fiber_vm.handle = Some(handle);
        let res = match handle.kind() {
            FiberKind::Fiber(context) => fiber_vm.run_context(*context),
            FiberKind::Enum(info) => Self::enumerator_fiber(&mut fiber_vm, info),
        };
        #[cfg(any(feature = "trace", feature = "trace-func"))]
        println!("<=== yield {:?} and terminate fiber.", res);
        let res = match res {
            Err(err) => match &err.kind {
                RubyErrorKind::MethodReturn(_) => Err(err.conv_localjump_err()),
                _ => Err(err),
            },
            res => res,
        };
        Box::into_raw(Box::new(res))
    }

    /// This BuiltinFunc is called in the fiber thread of a enumerator.
    /// `vm`: VM of created fiber.
    pub fn enumerator_fiber(vm: &mut VM, info: &EnumInfo) -> VMResult {
        let method = vm.get_method_from_receiver(info.receiver, info.method)?;
        let context = Context::new_noiseq();
        vm.context_push(ContextRef::from_ref(&context));
        vm.invoke_method(method, info.receiver, None, &info.args)?;
        vm.context_pop();
        Err(RubyError::stop_iteration("Iteration reached an end."))
    }
}

impl FiberContext {
    /// Resume child fiber.
    pub fn resume(&mut self, val: Value) -> VMResult {
        #[cfg(any(feature = "trace", feature = "trace-func"))]
        println!("===> resume");
        let ptr = self as _;
        match self.state {
            FiberState::Dead => Err(RubyError::fiber("Dead fiber called.")),
            FiberState::Created => {
                self.state = FiberState::Running;
                unsafe { *Box::from_raw(asm::invoke_context(ptr, val.get())) }
            }
            FiberState::Running => unsafe { *Box::from_raw(asm::switch_context(ptr, val.get())) },
        }
    }
}
