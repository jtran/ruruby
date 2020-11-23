use crate::*;

pub fn init(globals: &mut Globals) -> Value {
    let mut struct_class = Value::class_from(globals.builtins.object);
    struct_class.add_builtin_class_method("new", struct_new);
    struct_class
}

fn struct_new(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    args.check_args_min(1)?;
    let mut i = 0;
    let name = match args[0].as_string() {
        None => None,
        Some(s) => {
            match s.chars().nth(0) {
                Some(c) if c.is_ascii_uppercase() => {}
                _ => {
                    return Err(VM::error_name(format!(
                        "Identifier `{}` needs to be constant.",
                        s
                    )))
                }
            };
            i = 1;
            let s = IdentId::get_id(&format!("Struct::{}", s));
            Some(s)
        }
    };

    let mut class_val = Value::class_from(self_val);
    let class = class_val.as_mut_class();
    class.set_name(name);
    class.add_builtin_method_by_str("initialize", initialize);
    class.add_builtin_method_by_str("inspect", inspect);
    class_val.add_builtin_class_method("[]", builtin::class::new);
    class_val.add_builtin_class_method("new", builtin::class::new);

    let mut attr_args = Args::new(args.len() - i);
    let mut vec = vec![];
    for index in i..args.len() {
        let v = args[index];
        if v.as_symbol().is_none() {
            return Err(VM::error_type(format!(
                "{:?} is not a symbol.",
                args[index]
            )));
        };
        vec.push(v);
        attr_args[index - i] = v;
    }
    class_val.set_var_by_str("/members", Value::array_from(vec));
    builtin::module::attr_accessor(vm, class_val, &attr_args)?;

    match &args.block {
        Some(method) => {
            vm.class_push(class_val);
            let arg = Args::new1(class_val);
            vm.eval_block_self(method, class_val, &arg)?;
            vm.class_pop();
        }
        None => {}
    };
    Ok(class_val)
}

fn initialize(_: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    let class = self_val.get_class();
    let name = class.get_var(IdentId::get_id("/members")).unwrap();
    let members = name.as_array().unwrap();
    if members.elements.len() < args.len() {
        return Err(VM::error_argument("Struct size differs."));
    };
    for (i, arg) in args.iter().enumerate() {
        let id = members.elements[i].as_symbol().unwrap();
        let var = format!("@{:?}", id);
        self_val.set_var_by_str(&var, *arg);
    }
    Ok(Value::nil())
}

use std::borrow::Cow;
fn inspect(vm: &mut VM, self_val: Value, _args: &Args) -> VMResult {
    let mut inspect = format!("#<struct ");
    match self_val.get_class().as_class().name() {
        Some(id) => inspect += &IdentId::get_ident_name(id),
        None => {}
    };
    let name = match self_val.get_class().get_var(IdentId::get_id("/members")) {
        Some(name) => name,
        None => return Err(VM::error_internal("No /members.")),
    };
    //eprintln!("{:?}", name);
    let members = match name.as_array() {
        Some(aref) => aref,
        None => {
            return Err(VM::error_internal(format!(
                "Illegal _members value. {:?}",
                name
            )))
        }
    };

    for x in &members.elements {
        let id = IdentId::add_prefix(x.as_symbol().unwrap(), "@");
        let val = match self_val.get_var(id) {
            Some(v) => Cow::from(vm.val_inspect(v)?),
            None => Cow::from("nil"),
        };
        inspect = format!("{} {:?}={}", inspect, id, val);
    }
    inspect += ">";

    Ok(Value::string_from_string(inspect))
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[test]
    fn struct_test() {
        let program = r#"
        Customer = Struct.new(:name, :address) do
            def greeting
                "Hello #{name}!"
            end
        end
        assert "Hello Dave!", Customer.new("Dave", "123 Main").greeting
        assert "Hello Gave!", Customer["Gave", "456 Sub"].greeting
        "#;
        assert_script(program);
    }

    #[test]
    fn struct_inspect() {
        let program = r###"
        S = Struct.new(:a,:b)
        s = S.new(100,200)
        assert 100, s.a
        assert 200, s.b
        assert "#<struct S @a=100 @b=200>", s.inspect
        "###;
        assert_script(program);
    }
}
