#[cfg(feature = "perf")]
use crate::vm::perf::Perf;
use crate::*;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;

#[derive(Debug)]
pub struct FiberInfo {
    vm: VMRef,
    context: ContextRef,
    rec: Receiver<VMResult>,
    tx: SyncSender<usize>,
}

pub type FiberRef = Ref<FiberInfo>;

impl FiberInfo {
    pub fn new(
        vm: VMRef,
        context: ContextRef,
        rec: Receiver<VMResult>,
        tx: SyncSender<usize>,
    ) -> Self {
        FiberInfo {
            vm,
            context,
            rec,
            tx,
        }
    }
}

impl GC for FiberInfo {
    fn mark(&self, alloc: &mut Allocator) {
        self.vm.mark(alloc);
        self.context.mark(alloc);
    }
}

pub fn init_fiber(globals: &mut Globals) -> Value {
    let id = IdentId::get_ident_id("Fiber");
    let class = ClassRef::from(id, globals.builtins.object);
    let val = Value::class(globals, class);
    globals.add_builtin_instance_method(class, "inspect", inspect);
    globals.add_builtin_instance_method(class, "resume", resume);
    globals.add_builtin_class_method(val, "new", new);
    globals.add_builtin_class_method(val, "yield", yield_);
    val
}

// Class methods

fn new(vm: &mut VM, _: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0)?;
    let method = vm.expect_block(args.block)?;
    let mut context = vm.create_block_context(method)?;
    context.is_fiber = true;
    let (tx0, rx0) = std::sync::mpsc::sync_channel(0);
    let (tx1, rx1) = std::sync::mpsc::sync_channel(0);
    let new_fiber = VMRef::new(vm.dup_fiber(tx0, rx1));
    let val = Value::fiber(&vm.globals, new_fiber, context, rx0, tx1);
    Ok(val)
}

fn yield_(vm: &mut VM, _: Value, args: &Args) -> VMResult {
    let val = match args.len() {
        0 => Value::nil(),
        1 => args[0],
        _ => Value::array_from(&vm.globals, args.to_vec()),
    };
    if vm.parent_fiber.is_none() {
        return Err(vm.error_fiber("Can not yield from main fiber."));
    };
    #[cfg(feature = "perf")]
    #[cfg(not(tarpaulin_include))]
    vm.perf.get_perf(Perf::INVALID);
    vm.fiber_send_to_parent(Ok(val));
    Ok(Value::nil())
}

// Instance methods

fn inspect(vm: &mut VM, self_val: Value, _args: &Args) -> VMResult {
    let fref = vm.expect_fiber(self_val, "Expect Fiber.")?;
    let inspect = format!(
        "#<Fiber:0x{:<016x} ({:?})>",
        fref.id(),
        fref.vm.fiberstate(),
    );
    Ok(Value::string(&vm.globals, inspect))
}

fn resume(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0)?;
    let fiber = vm.expect_fiber(self_val, "")?;
    let mut context = fiber.context;
    context.is_fiber = true;
    let mut fiber_vm = fiber.vm;
    match fiber_vm.fiberstate() {
        FiberState::Dead => {
            return Err(vm.error_fiber("Dead fiber called."));
        }
        FiberState::Created => {
            fiber_vm.fiberstate_running();
            #[cfg(feature = "trace")]
            {
                println!("===> resume(spawn)");
            }
            let mut vm2 = fiber_vm;
            thread::spawn(move || vm2.run_context(context));
            #[cfg(feature = "perf")]
            #[cfg(not(tarpaulin_include))]
            vm.perf.get_perf(Perf::INVALID);
            let res = fiber.rec.recv().unwrap()?;
            return Ok(res);
        }
        FiberState::Running => {
            #[cfg(feature = "trace")]
            {
                println!("===> resume");
            }
            #[cfg(feature = "perf")]
            #[cfg(not(tarpaulin_include))]
            vm.perf.get_perf(Perf::INVALID);
            fiber.tx.send(1).unwrap();
            let res = fiber.rec.recv().unwrap()?;
            return Ok(res);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;
    #[test]
    fn fiber_test1() {
        let program = r#"
        def enum2gen(enum)
            Fiber.new do
                enum.each{|i|
                    Fiber.yield(i)
                }
            end
        end

        g = enum2gen(1..100)

        assert(1, g.resume)
        assert(2, g.resume)
        assert(3, g.resume)
        assert(4, g.resume)
        assert(5, g.resume)
        "#;
        assert_script(program);
    }

    #[test]
    fn fiber_test2() {
        let program = r#"
        f = Fiber.new do
            30.times {|x|
                Fiber.yield x
            }
        end
        assert(0, f.resume)
        assert(1, f.resume)
        assert(2, f.resume)
        assert(3, f.resume)
        assert(4, f.resume)
        "#;
        assert_script(program);
    }
}
