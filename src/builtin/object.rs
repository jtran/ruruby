use crate::*;

pub fn init_object(globals: &mut Globals) {
    let object = globals.object_class;
    globals.add_builtin_instance_method(object, "class", class);
    globals.add_builtin_instance_method(object, "object_id", object_id);
    globals.add_builtin_instance_method(object, "singleton_class", singleton_class);
    globals.add_builtin_instance_method(object, "inspect", inspect);
    globals.add_builtin_instance_method(object, "eql?", eql);
    globals.add_builtin_instance_method(object, "to_i", toi);
    globals.add_builtin_instance_method(object, "instance_variable_set", instance_variable_set);
    globals.add_builtin_instance_method(object, "instance_variable_get", instance_variable_get);
    globals.add_builtin_instance_method(object, "instance_variables", instance_variables);
    globals.add_builtin_instance_method(object, "freeze", freeze);
    globals.add_builtin_instance_method(object, "super", super_);
    globals.add_builtin_instance_method(object, "equal?", equal);
    globals.add_builtin_instance_method(object, "send", send);
    globals.add_builtin_instance_method(object, "yield", object_yield);
    globals.add_builtin_instance_method(object, "eval", eval);
}

fn class(vm: &mut VM, args: &Args) -> VMResult {
    let class = args.self_value.get_class_object(&vm.globals);
    Ok(class)
}

fn object_id(_vm: &mut VM, args: &Args) -> VMResult {
    let id = args.self_value.id();
    Ok(Value::fixnum(id as i64))
}

fn singleton_class(vm: &mut VM, args: &Args) -> VMResult {
    vm.get_singleton_class(args.self_value)
}

fn inspect(vm: &mut VM, args: &Args) -> VMResult {
    let inspect = vm.val_inspect(args.self_value);
    Ok(Value::string(&vm.globals, inspect))
}

fn eql(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1, 1)?;
    Ok(Value::bool(args.self_value == args[0]))
}

fn toi(vm: &mut VM, args: &Args) -> VMResult {
    //vm.check_args_num(args.len(), 1, 1)?;
    let self_ = args.self_value;
    let num = match &self_.as_rvalue() {
        Some(info) => match &info.kind {
            ObjKind::FixNum(val) => *val,
            ObjKind::FloatNum(val) => f64::trunc(*val) as i64,
            _ => return Err(vm.error_type("Must be a number.")),
        },
        None => {
            if self_.is_packed_num() {
                if self_.is_packed_fixnum() {
                    self_.as_packed_fixnum()
                } else {
                    f64::trunc(self_.as_packed_flonum()) as i64
                }
            } else {
                return Err(vm.error_type("Must be a number."));
            }
        }
    };
    Ok(Value::fixnum(num))
}

fn instance_variable_set(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 2, 2)?;
    let name = args[0];
    let val = args[1];
    let var_id = match name.as_symbol() {
        Some(symbol) => symbol,
        None => match name.as_string() {
            Some(s) => vm.globals.get_ident_id(s),
            None => return Err(vm.error_type("1st arg must be Symbol or String.")),
        },
    };
    let mut self_obj = args.self_value.as_object();
    self_obj.set_var(var_id, val);
    Ok(val)
}

fn instance_variable_get(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1, 1)?;
    let name = args[0];
    let var_id = match name.as_symbol() {
        Some(symbol) => symbol,
        None => match name.as_string() {
            Some(s) => vm.globals.get_ident_id(s),
            None => return Err(vm.error_type("1st arg must be Symbol or String.")),
        },
    };
    let self_obj = args.self_value.as_object();
    let val = match self_obj.get_var(var_id) {
        Some(val) => val,
        None => Value::nil(),
    };
    Ok(val)
}

fn instance_variables(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0, 0)?;
    let mut receiver = args.self_value.as_object();
    let res = receiver
        .var_table()
        .keys()
        .filter(|x| vm.globals.get_ident_name(**x).chars().nth(0) == Some('@'))
        .map(|x| Value::symbol(*x))
        .collect();
    Ok(Value::array_from(&vm.globals, res))
}

fn freeze(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0, 0)?;
    Ok(args.self_value)
}

fn super_(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0, 0)?;
    let context = vm.context();
    let iseq = context.iseq_ref;
    if let ISeqKind::Method(m) = iseq.kind {
        let class = match iseq.class_defined {
            Some(list) => list.class,
            None => {
                let inspect = vm.val_inspect(args.self_value);
                return Err(vm.error_nomethod(format!(
                    "no superclass method `{}' for {}.",
                    vm.globals.get_ident_name(m),
                    inspect,
                )));
            }
        };
        let method = match class.superclass() {
            Some(class) => vm.get_instance_method(class, m)?,
            None => {
                let inspect = vm.val_inspect(args.self_value);
                return Err(vm.error_nomethod(format!(
                    "no superclass method `{}' for {}.",
                    vm.globals.get_ident_name(m),
                    inspect,
                )));
            }
        };
        let param_num = iseq.param_ident.len();
        let mut args = Args::new0(context.self_value, None);
        for i in 0..param_num {
            args.push(context.get_lvar(LvarId::from_usize(i)));
        }
        let val = vm.eval_send(method, &args)?;
        Ok(val)
    } else {
        return Err(vm.error_nomethod("super called outside of method"));
    }
}

fn equal(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1, 1)?;
    Ok(Value::bool(args.self_value.id() == args[0].id()))
}

fn send(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1, 100)?;
    let receiver = args.self_value;
    let method_id = match args[0].as_symbol() {
        Some(symbol) => symbol,
        None => return Err(vm.error_argument("Must be a symbol.")),
    };
    let method = vm.get_method(receiver, method_id)?;

    let mut new_args = Args::new(args.len() - 1);
    for i in 0..args.len() - 1 {
        new_args[i] = args[i + 1];
    }
    new_args.self_value = args.self_value;
    new_args.block = args.block;
    let res = vm.eval_send(method, &new_args)?;
    Ok(res)
}

fn object_yield(vm: &mut VM, args: &Args) -> VMResult {
    let outer = vm.caller_context();
    let method = match vm.context().block {
        Some(block) => block,
        None => return Err(vm.error_argument("Yield needs block.")),
    };
    let args = args.clone();
    let iseq = vm.get_iseq(method)?;
    let res = vm.vm_run(iseq, Some(outer), &args)?;
    Ok(res)
}

fn eval(vm: &mut VM, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 1, 1)?;
    expect_string!(program, vm, args[0]);
    let method = vm.parse_program_eval(std::path::PathBuf::from("eval"), program)?;
    let context = vm.context();
    let args = Args::new0(context.self_value, None);
    let res = vm.eval_block(method, &args)?;
    Ok(res)
}

#[cfg(test)]
mod test {
    use crate::test::*;

    #[test]
    fn instance_variables() {
        let program = r#"
        obj = Object.new
        obj.instance_variable_set("@foo", "foo")
        obj.instance_variable_set(:@bar, 777)

        def ary_cmp(a,b)
            return false if a - b != []
            return false if b - a != []
            true
        end

        assert(true, ary_cmp([:@foo, :@bar], obj.instance_variables))
        "#;
        assert_script(program);
    }

    #[test]
    fn object_send() {
        let program = r#"
        class Foo
            def foo(); "foo" end
            def bar(); "bar" end
            def baz(); "baz" end
        end

        # 任意のキーとメソッド(の名前)の関係をハッシュに保持しておく
        # レシーバの情報がここにはないことに注意
        methods = {1 => :foo, 2 => :bar, 3 => :baz}

        # キーを使って関連するメソッドを呼び出す
        # レシーバは任意(Foo クラスのインスタンスである必要もない)
        assert "foo", Foo.new.send(methods[1])
        assert "bar", Foo.new.send(methods[2])
        assert "baz", Foo.new.send(methods[3])
        "#;
        assert_script(program);
    }

    #[test]
    fn object_yield() {
        let program = r#"
        # ブロック付きメソッドの定義、
        # その働きは与えられたブロック(手続き)に引数1, 2を渡して実行すること
        def foo
            yield(1,2)
        end

        # fooに「2引数手続き、その働きは引数を配列に括ってpで印字する」というものを渡して実行させる
        assert [1, 2], foo {|a,b| [a, b]}  # => [1, 2] (要するに p [1, 2] を実行した)
        # 今度は「2引数手続き、その働きは足し算をしてpで印字する」というものを渡して実行させる
        assert 3, foo {|a, b| p a + b}  # => 3 (要するに p 1 + 2 を実行した)

        # 今度のブロック付きメソッドの働きは、
        # 与えられたブロックに引数10を渡して起動し、続けざまに引数20を渡して起動し、
        # さらに引数30を渡して起動すること
        def bar
            a = []
            a << yield(10)
            a << yield(20)
            a << yield(30)
        end

        # barに「1引数手続き、その働きは引数に3を足してpで印字する」というものを渡して実行させる
        assert [13, 23, 33], bar {|v| v + 3 }
        # => 13
        #    23
        #    33 (同じブロックが3つのyieldで3回起動された。
        #        具体的には 10 + 3; 20 + 3; 30 + 3 を実行した)

        "#;
        assert_script(program);
    }

    #[test]
    fn object_eval() {
        let program = r#"
        a = 100
        eval("b = 100; assert(100, b);")
        assert(77, eval("a = 77"))
        assert(77, a)
        "#;
        assert_script(program);
    }

    #[test]
    fn object_yield2() {
        let program = r#"
        class Array
            def iich
                len = self.size
                for i in 0...len
                    puts self[i]
                    yield self[i]
                end
            end
        end

        sum = 0
        [1,2,3,4,5].iich{|x| sum += x }
        assert(15 ,sum)
        "#;
        assert_script(program);
    }
}
