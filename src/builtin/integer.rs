use crate::*;

pub fn init(_globals: &mut Globals) -> Value {
    let object = BuiltinClass::object();
    let numeric = object.get_var_by_str("Numeric").unwrap();

    let mut class = ClassRef::from_str("Integer", numeric);
    class.add_builtin_method_by_str("+", add);
    class.add_builtin_method_by_str("-", sub);
    class.add_builtin_method_by_str("*", mul);
    class.add_builtin_method_by_str("div", quotient);
    class.add_builtin_method_by_str("==", eq);
    class.add_builtin_method_by_str("!=", neq);
    class.add_builtin_method_by_str(">=", ge);
    class.add_builtin_method_by_str(">", gt);
    class.add_builtin_method_by_str("<=", le);
    class.add_builtin_method_by_str("<", lt);
    class.add_builtin_method_by_str("<=>", cmp);

    class.add_builtin_method_by_str("times", times);
    class.add_builtin_method_by_str("upto", upto);
    class.add_builtin_method_by_str("step", step);
    class.add_builtin_method_by_str("chr", chr);
    class.add_builtin_method_by_str("to_f", tof);
    class.add_builtin_method_by_str("to_i", toi);
    class.add_builtin_method_by_str("floor", floor);
    class.add_builtin_method_by_str("even?", even);
    Value::class(class)
}

// Class methods

// Instance methods

fn add(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1)?;
    let lhs = self_val.to_real().unwrap();
    match args[0].to_real() {
        Some(rhs) => Ok((lhs + rhs).to_val()),
        None => match args[0].to_complex() {
            Some((r, i)) => {
                let r = lhs + r;
                let i = i;
                Ok(Value::complex(r.to_val(), i.to_val()))
            }
            None => Err(vm.error_undefined_op("+", args[0], self_val)),
        },
    }
}

fn sub(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1)?;
    let lhs = self_val.to_real().unwrap();
    match args[0].to_real() {
        Some(rhs) => Ok((lhs - rhs).to_val()),
        None => match args[0].to_complex() {
            Some((r, i)) => {
                let r = lhs - r;
                let i = -i;
                Ok(Value::complex(r.to_val(), i.to_val()))
            }
            None => Err(vm.error_undefined_op("-", args[0], self_val)),
        },
    }
}

fn mul(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1)?;
    let lhs = self_val.to_real().unwrap();
    match args[0].to_real() {
        Some(rhs) => Ok((lhs * rhs).to_val()),
        None => match args[0].to_complex() {
            Some((r, i)) => {
                let r = lhs * r;
                let i = lhs * i;
                Ok(Value::complex(r.to_val(), i.to_val()))
            }
            None => Err(vm.error_undefined_op("-", args[0], self_val)),
        },
    }
}

fn quotient(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1)?;
    let lhs = self_val.to_real().unwrap();
    match args[0].to_real() {
        Some(rhs) => Ok((lhs.quo(rhs)).to_val()),
        None => Err(vm.error_undefined_op("div", args[0], self_val)),
    }
}

fn eq(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1)?;
    let lhs = self_val.expect_integer(vm, "Receiver")?;
    match args[0].unpack() {
        RV::Integer(rhs) => Ok(Value::bool(lhs == rhs)),
        RV::Float(rhs) => Ok(Value::bool(lhs as f64 == rhs)),
        _ => Ok(Value::bool(false)),
    }
}

fn neq(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1)?;
    let lhs = self_val.expect_integer(vm, "Receiver")?;
    match args[0].unpack() {
        RV::Integer(rhs) => Ok(Value::bool(lhs != rhs)),
        RV::Float(rhs) => Ok(Value::bool(lhs as f64 != rhs)),
        _ => Ok(Value::bool(true)),
    }
}

macro_rules! define_cmp {
    ($vm:ident, $self_val:ident, $args:ident, $op:ident) => {
        $vm.check_args_num($args.len(), 1)?;
        let lhs = $self_val.expect_integer($vm, "Receiver")?;
        match $args[0].unpack() {
            RV::Integer(rhs) => return Ok(Value::bool(lhs.$op(&rhs))),
            RV::Float(rhs) => return Ok(Value::bool((lhs as f64).$op(&rhs))),
            _ => {
                return Err($vm.error_argument(format!(
                    "Comparison of Integer with {} failed.",
                    $args[0].get_class_name()
                )))
            }
        }
    };
}

fn ge(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    define_cmp!(vm, self_val, args, ge);
}

fn gt(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    define_cmp!(vm, self_val, args, gt);
}

fn le(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    define_cmp!(vm, self_val, args, le);
}

fn lt(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    define_cmp!(vm, self_val, args, lt);
}

fn cmp(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    //use std::cmp::Ordering;
    vm.check_args_num(args.len(), 1)?;
    let lhs = self_val.expect_integer(vm, "Receiver")?;
    let res = match args[0].unpack() {
        RV::Integer(rhs) => lhs.partial_cmp(&rhs),
        RV::Float(rhs) => (lhs as f64).partial_cmp(&rhs),
        _ => return Ok(Value::nil()),
    };
    match res {
        Some(ord) => Ok(Value::integer(ord as i64)),
        None => Ok(Value::nil()),
    }
}

fn times(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0)?;
    let method = match args.block {
        Some(method) => method,
        None => {
            let id = IdentId::get_id("times");
            let val = vm.create_enumerator(id, self_val, args.clone())?;
            return Ok(val);
        }
    };
    let num = self_val.as_integer().unwrap();
    if num < 1 {
        return Ok(self_val);
    };
    let mut arg = Args::new1(Value::nil());
    for i in 0..num {
        arg[0] = Value::integer(i);
        vm.eval_block(method, &arg)?;
    }
    Ok(self_val)
}

fn upto(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1)?;
    let method = match args.block {
        Some(method) => method,
        None => {
            let id = IdentId::get_id("upto");
            let val = vm.create_enumerator(id, self_val, args.clone())?;
            return Ok(val);
        }
    };
    let num = self_val.as_integer().unwrap();
    let max = args[0].expect_integer(vm, "Arg")?;
    if num <= max {
        let mut arg = Args::new1(Value::nil());
        for i in num..max + 1 {
            arg[0] = Value::integer(i);
            vm.eval_block(method, &arg)?;
        }
    }
    Ok(self_val)
}

fn step(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_range(args.len(), 1, 2)?;
    let method = match args.block {
        Some(method) => method,
        None => {
            let id = IdentId::get_id("step");
            let val = vm.create_enumerator(id, self_val, args.clone())?;
            return Ok(val);
        }
    };
    let start = self_val.expect_integer(vm, "Start")?;
    let limit = args[0].expect_integer(vm, "Limit")?;
    let step = if args.len() == 2 {
        let step = args[1].expect_integer(vm, "Step")?;
        if step == 0 {
            return Err(vm.error_argument("Step can not be 0."));
        }
        step
    } else {
        1
    };

    let mut arg = Args::new1(Value::nil());
    let mut i = start;
    loop {
        if step > 0 && i > limit || step < 0 && limit > i {
            break;
        }
        arg[0] = Value::integer(i);
        vm.eval_block(method, &arg)?;
        i += step;
    }

    Ok(self_val)
}

/// Built-in function "chr".
fn chr(vm: &mut VM, self_val: Value, _: &Args) -> VMResult {
    let num = self_val.as_integer().unwrap();
    if 0 > num || num > 255 {
        return Err(vm.error_unimplemented("Currently, receiver must be 0..255."));
    };
    Ok(Value::bytes(vec![num as u8]))
}

fn floor(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0)?;
    self_val.as_integer().unwrap();
    Ok(self_val)
}

fn tof(_vm: &mut VM, self_val: Value, _: &Args) -> VMResult {
    let num = self_val.as_integer().unwrap();
    Ok(Value::float(num as f64))
}

fn toi(_vm: &mut VM, self_val: Value, _: &Args) -> VMResult {
    //vm.check_args_num(args.len(), 1, 1)?;
    let num = self_val.as_integer().unwrap();
    Ok(Value::integer(num))
}

fn even(_vm: &mut VM, self_val: Value, _: &Args) -> VMResult {
    let num = self_val.as_integer().unwrap();
    Ok(Value::bool(num % 2 == 0))
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[test]
    fn integer1() {
        let program = r#"
        assert(4.0, 4.to_f)
        assert(-4.0, -4.to_f)
        assert(4, 4.floor)
        assert(-4, -4.floor)
        assert(true, 8.even?)
        assert(false, 9.even?)
        "#;
        assert_script(program);
    }

    #[test]
    fn integer_cmp() {
        let program = r#"
            assert true, 4.send(:"==", 4)
            assert false, 4.send(:"==", 14)
            assert false, 4.send(:"==", "4")
            assert true, 4.send(:"==", 4.0)
            assert false, 4.send(:"==", 4.1)

            assert false, 4.send(:"!=", 4)
            assert true, 4.send(:"!=", 14)
            assert true, 4.send(:"!=", "4")
            assert false, 4.send(:"!=", 4.0)
            assert true, 4.send(:"!=", 4.1)

            assert true, 4.send(:">=", -1)
            assert true, 4.send(:">=", 4)
            assert false, 4.send(:">=", 14)
            #assert false, 4.send(:">=", "4")
            assert true, 4.send(:">=", 3.9)
            assert true, 4.send(:">=", 4.0)
            assert false, 4.send(:">=", 4.1)

            assert false, 4.send(:"<=", -1)
            assert true, 4.send(:"<=", 4)
            assert true, 4.send(:"<=", 14)
            #assert false, 4.send(:"<=", "4")
            assert false, 4.send(:"<=", 3.9)
            assert true, 4.send(:"<=", 4.0)
            assert true, 4.send(:"<=", 4.1)

            assert true, 4.send(:">", -1)
            assert false, 4.send(:">", 4)
            assert false, 4.send(:">", 14)
            #assert false, 4.send(:">", "4")
            assert true, 4.send(:">", 3.9)
            assert false, 4.send(:">", 4.0)
            assert false, 4.send(:">", 4.1)

            assert false, 4.send(:"<", -1)
            assert false, 4.send(:"<", 4)
            assert true, 4.send(:"<", 14)
            #assert false, 4.send(:"<", "4")
            assert false, 4.send(:"<", 3.9)
            assert false, 4.send(:"<", 4.0)
            assert true, 4.send(:"<", 4.1)

            assert(0, 3.send(:"<=>", 3))
            assert(1, 5.send(:"<=>", 3))
            assert(-1, 3.send(:"<=>", 5))
            assert(0, 3.send(:"<=>", 3.0))
            assert(1, 5.send(:"<=>", 3.9))
            assert(-1, 3.send(:"<=>", 5.8))
            assert(nil, 3.send(:"<=>", "three"))

            assert(0, 3 <=> 3)
            assert(1, 5 <=> 3)
            assert(-1, 3 <=> 5)
            assert(0, 3 <=> 3.0)
            assert(1, 5 <=> 3.9)
            assert(-1, 3 <=> 5.8)
            assert(nil, 3 <=> "three")
        "#;
        assert_script(program);
    }

    #[test]
    fn integer_times() {
        let program = r#"
        res = []
        assert 5, 5.times {|x| res[x] = x * x}
        assert [0,1,4,9,16], res
        res = []
        assert 0, 0.times {|x| res[x] = x * x}
        assert [], res
        "#;
        assert_script(program);
    }

    #[test]
    fn integer_upto() {
        let program = r#"
        res = []
        assert 5, 5.upto(8) {|x| res << x * x}
        assert [25, 36, 49, 64], res
        res = []
        assert 5, 5.upto(4) {|x| res << x * x}
        assert [], res
        enum = 5.upto(8)
        assert [10, 12, 14, 16], enum.map{|x| x * 2}
        "#;
        assert_script(program);
    }

    #[test]
    fn integer_step() {
        let program = r#"
        res = 0
        4.step(20){|x| res += x}
        assert(204, res)
        res = 0
        4.step(20, 3){|x| res += x}
        assert(69, res)
        "#;
        assert_script(program);
    }

    #[test]
    fn integer_quotient() {
        let program = r#"
        assert(1, 3.div(2))
        assert(1, 3.div(2.0))
        assert(-2, (-3).div(2))
        assert(-2, (-3).div(2.0))
        "#;
        assert_script(program);
    }
}
