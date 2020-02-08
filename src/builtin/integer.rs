use crate::vm::*;

pub fn init_integer(globals: &mut Globals) -> PackedValue {
    let id = globals.get_ident_id("Integer");
    let class = ClassRef::from(id, globals.object);
    globals.add_builtin_instance_method(class, "times", integer_times);
    globals.add_builtin_instance_method(class, "chr", integer_chr);
    globals.add_builtin_instance_method(class, "to_f", integer_tof);
    PackedValue::class(globals, class)
}

// Class methods

// Instance methods

fn integer_times(vm: &mut VM, args: &Args, block: Option<MethodRef>) -> VMResult {
    let num = args.self_value.as_fixnum().unwrap();
    if num < 1 {
        return Ok(PackedValue::nil());
    };
    match block {
        None => return Ok(PackedValue::nil()),
        Some(method) => {
            let context = vm.context();
            let self_value = context.self_value;
            let info = vm.globals.get_method_info(method);
            let iseq = info.as_iseq(&vm)?;
            let mut arg = Args::new1(self_value, None, PackedValue::nil());
            for i in 0..num {
                arg[0] = PackedValue::fixnum(i);
                vm.vm_run(iseq, Some(context), &arg, None, None)?;
                vm.stack_pop();
            }
        }
    }
    Ok(args.self_value)
}

/// Built-in function "chr".
fn integer_chr(_vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
    let num = args.self_value.as_fixnum().unwrap();
    Ok(Value::Char(num as u8).pack())
}

fn integer_tof(_vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
    let num = args.self_value.as_fixnum().unwrap();
    Ok(PackedValue::flonum(num as f64))
}
