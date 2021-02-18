use crate::*;

pub fn init() {
    let io_class = Module::class_under_object();
    io_class.add_builtin_method_by_str("<<", output);
    io_class.add_builtin_method_by_str("isatty", isatty);
    io_class.add_builtin_method_by_str("tty?", isatty);
    io_class.add_builtin_method_by_str("flush", flush);
    BuiltinClass::set_toplevel_constant("IO", io_class);
    let stdout = Value::ordinary_object(io_class);
    BuiltinClass::set_toplevel_constant("STDOUT", stdout);
}

use std::io::{self, Write};

fn output(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    args.check_args_num(1)?;
    match args[0].as_string() {
        Some(s) => print!("{}", s),
        None => {
            let s = args[0].val_to_s(vm)?;
            print!("{}", s)
        }
    };
    io::stdout().flush().unwrap();
    Ok(self_val)
}

fn isatty(_: &mut VM, _: Value, args: &Args) -> VMResult {
    args.check_args_num(0)?;
    Ok(Value::true_val())
}

fn flush(_: &mut VM, self_val: Value, args: &Args) -> VMResult {
    args.check_args_num(0)?;
    io::stdout().flush().unwrap();
    Ok(self_val)
}
