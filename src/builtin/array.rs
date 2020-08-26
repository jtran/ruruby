use crate::error::RubyError;
use crate::*;

pub fn init(_globals: &mut Globals) -> Value {
    let array_id = IdentId::get_id("Array");
    let mut class = ClassRef::from(array_id, BuiltinClass::object());
    let mut class_obj = Value::class(class);
    class.add_builtin_instance_method("inspect", inspect);
    class.add_builtin_instance_method("to_s", inspect);
    class.add_builtin_instance_method("length", length);
    class.add_builtin_instance_method("size", length);
    class.add_builtin_instance_method("empty?", empty);
    class.add_builtin_instance_method("[]=", set_elem);
    class.add_builtin_instance_method("push", push);
    class.add_builtin_instance_method("<<", push);
    class.add_builtin_instance_method("pop", pop);
    class.add_builtin_instance_method("*", mul);
    class.add_builtin_instance_method("+", add);
    class.add_builtin_instance_method("-", sub);
    class.add_builtin_instance_method("<=>", cmp);

    class.add_builtin_instance_method("shift", shift);
    class.add_builtin_instance_method("unshift", unshift);

    class.add_builtin_instance_method("concat", concat);
    class.add_builtin_instance_method("map", map);
    class.add_builtin_instance_method("flat_map", flat_map);
    class.add_builtin_instance_method("each", each);

    class.add_builtin_instance_method("include?", include);
    class.add_builtin_instance_method("reverse", reverse);
    class.add_builtin_instance_method("reverse!", reverse_);
    class.add_builtin_instance_method("rotate!", rotate_);

    class.add_builtin_instance_method("transpose", transpose);
    class.add_builtin_instance_method("min", min);
    class.add_builtin_instance_method("fill", fill);
    class.add_builtin_instance_method("clear", clear);
    class.add_builtin_instance_method("uniq!", uniq_);
    class.add_builtin_instance_method("uniq", uniq);
    class.add_builtin_instance_method("any?", any_);
    class.add_builtin_instance_method("all?", all_);

    class.add_builtin_instance_method("slice!", slice_);
    class.add_builtin_instance_method("max", max);
    class.add_builtin_instance_method("first", first);
    class.add_builtin_instance_method("last", last);
    class.add_builtin_instance_method("dup", dup);
    class.add_builtin_instance_method("clone", dup);
    class.add_builtin_instance_method("pack", pack);
    class.add_builtin_instance_method("join", join);
    class.add_builtin_instance_method("drop", drop);
    class.add_builtin_instance_method("zip", zip);
    class.add_builtin_instance_method("grep", grep);
    class.add_builtin_instance_method("sort", sort);
    class.add_builtin_instance_method("count", count);

    class_obj.add_builtin_class_method("new", array_new);
    class_obj
}

// Class methods

fn array_new(vm: &mut VM, _: Value, args: &Args) -> VMResult {
    vm.check_args_range(args.len(), 0, 2)?;
    let array_vec = match args.len() {
        0 => vec![],
        1 => match args[0].unpack() {
            RV::Integer(num) if num >= 0 => vec![Value::nil(); num as usize],
            RV::Object(oref) => match &oref.kind {
                ObjKind::Array(aref) => aref.elements.clone(),
                _ => return Err(vm.error_argument("Invalid arguments")),
            },
            _ => return Err(vm.error_argument("Invalid arguments")),
        },
        2 => {
            let arg_num = args[0]
                .as_integer()
                .ok_or(vm.error_argument("Invalid arguments"))?;
            vec![args[1]; arg_num as usize]
        }
        _ => unreachable!(),
    };
    let array = Value::array_from(array_vec);
    Ok(array)
}

// Instance methods

fn inspect(vm: &mut VM, self_val: Value, _args: &Args) -> VMResult {
    let aref = self_val.as_array().unwrap();
    let s = aref.to_s(vm);
    Ok(Value::string(s))
}

fn set_elem(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    let aref = self_val.as_mut_array().unwrap();
    let val = aref.set_elem(vm, args)?;
    Ok(val)
}

fn cmp(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 1)?;
    let lhs = &self_val.as_array().unwrap().elements;
    let rhs = &(match args[0].as_array() {
        Some(aref) => aref,
        None => return Ok(Value::nil()),
    }
    .elements);
    if lhs.len() >= rhs.len() {
        for (i, rhs_v) in rhs.iter().enumerate() {
            match vm.eval_compare(*rhs_v, lhs[i])?.as_integer() {
                Some(0) => {}
                Some(ord) => return Ok(Value::integer(ord)),
                None => return Ok(Value::nil()),
            }
        }
        if lhs.len() == rhs.len() {
            Ok(Value::integer(0))
        } else {
            Ok(Value::integer(1))
        }
    } else {
        for (i, lhs_v) in lhs.iter().enumerate() {
            match vm.eval_compare(rhs[i], *lhs_v)?.as_integer() {
                Some(0) => {}
                Some(ord) => return Ok(Value::integer(ord)),
                None => return Ok(Value::nil()),
            }
        }
        Ok(Value::integer(-1))
    }
}

fn push(_vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    let aref = self_val.as_mut_array().unwrap();
    for arg in args.iter() {
        aref.elements.push(*arg);
    }
    Ok(self_val)
}

fn pop(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_mut_array().unwrap();
    let res = aref.elements.pop().unwrap_or_default();
    Ok(res)
}

fn shift(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_range(args.len(), 0, 1)?;
    let mut array_flag = false;
    let num = if args.len() == 0 {
        0
    } else {
        let i = args[0].expect_integer(vm, "1st arg")?;
        if i < 0 {
            return Err(vm.error_argument("Negative array size."));
        }
        array_flag = true;
        i as usize
    };

    let mut aref = self_val.as_mut_array().unwrap();
    if array_flag {
        if aref.elements.len() < num {
            return Ok(Value::array_from(vec![]));
        }
        let new = aref.elements.split_off(num);
        let res = aref.elements[0..num].to_vec();
        aref.elements = new;
        Ok(Value::array_from(res))
    } else {
        if aref.elements.len() == 0 {
            return Ok(Value::nil());
        }
        let new = aref.elements.split_off(1);
        let res = aref.elements[0];
        aref.elements = new;
        Ok(res)
    }
}

fn unshift(_vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    if args.len() == 0 {
        return Ok(self_val);
    }
    let mut new = args[0..args.len()].to_owned();
    let mut aref = self_val.as_mut_array().unwrap();
    new.append(&mut aref.elements);
    aref.elements = new;
    Ok(self_val)
}

fn length(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    let res = Value::integer(aref.elements.len() as i64);
    Ok(res)
}

fn empty(_vm: &mut VM, self_val: Value, _args: &Args) -> VMResult {
    let aref = self_val.as_array().unwrap();
    let res = Value::bool(aref.elements.is_empty());
    Ok(res)
}

fn mul(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 1)?;
    let aref = self_val.as_array().unwrap();
    if let Some(num) = args[0].as_integer() {
        let v = match num {
            i if i < 0 => return Err(vm.error_argument("Negative argument.")),
            0 => vec![],
            1 => aref.elements.clone(),
            _ => {
                let len = aref.elements.len();
                let src = &aref.elements[0..len];
                let mut v = vec![Value::nil(); len * num as usize];
                //println!("dest:{:?} src:{:?}", aref.elements, src);
                let mut i = 0;
                for _ in 0..num {
                    for src_val in src[0..len].iter() {
                        v[i] = src_val.dup();
                        i += 1;
                    }
                }
                v
            }
        };
        let res = Value::array_from(v);
        Ok(res)
    } else if let Some(s) = args[0].as_string() {
        match aref.elements.len() {
            0 => return Ok(Value::string("".to_string())),
            1 => {
                let res = vm.val_to_s(aref.elements[0]);
                return Ok(Value::string(res));
            }
            _ => {
                let mut res = vm.val_to_s(aref.elements[0]);
                for i in 1..aref.elements.len() {
                    res = format!("{}{}{}", res, s, vm.val_to_s(aref.elements[i]));
                }
                return Ok(Value::string(res));
            }
        };
    } else {
        return Err(vm.error_undefined_op("*", args[0], self_val));
    }
}

fn add(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    let mut lhs = self_val.expect_array(vm, "Receiver")?.elements.clone();
    let mut arg0 = args[0];
    let mut rhs = arg0.expect_array(vm, "Argument")?.elements.clone();
    lhs.append(&mut rhs);
    Ok(Value::array_from(lhs))
}

fn concat(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    let lhs = self_val.as_mut_array().unwrap();
    let mut arg0 = args[0];
    let mut rhs = arg0.expect_array(vm, "Argument")?.elements.clone();
    lhs.elements.append(&mut rhs);
    Ok(self_val)
}

fn sub(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    let lhs_v = &self_val.expect_array(vm, "Receiver")?.elements;
    let mut arg0 = args[0];
    let rhs_v = &arg0.expect_array(vm, "Argument")?.elements;
    let mut v = vec![];
    for lhs in lhs_v {
        let mut flag = true;
        for rhs in rhs_v {
            if vm.eval_eq(*lhs, *rhs) {
                flag = false;
                break;
            }
        }
        if flag {
            v.push(*lhs)
        }
    }
    Ok(Value::array_from(v))
}

fn map(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    let method = match args.block {
        Some(method) => method,
        None => {
            let id = IdentId::get_id("map");
            let val = vm.create_enumerator(id, self_val, args.clone())?;
            return Ok(val);
        }
    };

    let mut args = Args::new1(Value::nil());

    let mut res = vec![];
    for elem in &aref.elements {
        args[0] = *elem;
        let val = vm.eval_block(method, &args)?;
        vm.temp_push(val);
        res.push(val);
    }

    let res = Value::array_from(res);
    Ok(res)
}

fn flat_map(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    let method = vm.expect_block(args.block)?;
    let param_num = vm.get_iseq(method)?.params.req_params;
    let mut arg = Args::new(param_num);

    let aref = self_val.as_mut_array().unwrap();
    let mut res = vec![];
    for elem in &mut aref.elements {
        if param_num == 0 {
        } else if param_num == 1 {
            arg[0] = *elem;
        } else {
            match elem.as_array() {
                Some(ary) => {
                    for i in 0..param_num {
                        arg[i] = ary.elements[i];
                    }
                }
                None => arg[0] = *elem,
            }
        }

        let mut ary = vm.eval_block(method, &arg)?;
        vm.temp_push(ary);
        match ary.as_mut_array() {
            Some(ary) => {
                res.append(&mut ary.elements);
            }
            None => res.push(ary),
        }
    }
    let res = Value::array_from(res);
    Ok(res)
}

fn each(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;

    let method = match args.block {
        Some(method) => method,
        None => {
            let id = IdentId::get_id("each");
            let val = vm.create_enumerator(id, self_val, args.clone())?;
            return Ok(val);
        }
    };

    let aref = self_val.as_mut_array().unwrap();
    let mut arg = Args::new(vm.get_iseq(method)?.params.req_params);
    for i in &mut aref.elements {
        match i.as_array() {
            Some(aref) if arg.len() != 1 => {
                for j in 0..arg.len() {
                    arg[j] = if j < aref.elements.len() {
                        aref.elements[j]
                    } else {
                        Value::nil()
                    };
                }
            }
            _ => {
                arg[0] = *i;
                for j in 1..arg.len() {
                    arg[j] = Value::nil();
                }
            }
        };

        vm.eval_block(method, &arg)?;
    }
    Ok(self_val)
}

fn include(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    let target = args[0];
    let aref = self_val.as_array().unwrap();
    for item in aref.elements.iter() {
        if vm.eval_eq(*item, target) {
            return Ok(Value::true_val());
        }
    }
    Ok(Value::false_val())
}

fn reverse(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    let mut res = aref.elements.clone();
    res.reverse();
    Ok(Value::array_from(res))
}

fn reverse_(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_mut_array().unwrap();
    aref.elements.reverse();
    Ok(self_val)
}

fn rotate_(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_range(args.len(), 0, 1)?;
    let i = if args.len() == 0 {
        1
    } else {
        match args[0].as_integer() {
            Some(i) => i,
            None => return Err(vm.error_argument("Must be Integer.")),
        }
    };
    let mut aref = self_val.as_mut_array().unwrap();
    if i == 0 {
        Ok(self_val)
    } else if i > 0 {
        let i = i % (aref.elements.len() as i64);
        let mut vec = &mut aref.clone().elements;
        let mut vec2 = vec.split_off(i as usize);
        vec2.append(&mut vec);
        aref.elements = vec2;
        Ok(self_val)
    } else {
        let len = aref.elements.len() as i64;
        let i = (-i) % len;
        let mut vec = &mut aref.clone().elements;
        let mut vec2 = vec.split_off((len - i) as usize);
        vec2.append(&mut vec);
        aref.elements = vec2;
        Ok(self_val)
    }
}

fn transpose(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_mut_array().unwrap();
    if aref.elements.len() == 0 {
        return Ok(Value::array_from(vec![]));
    }
    let mut vec = vec![];
    for elem in &mut aref.elements {
        let ary = elem
            .as_array()
            .ok_or(vm.error_argument("Each element of receiver must be an array."))?
            .elements
            .clone();
        vec.push(ary);
    }
    let len = vec[0].len();
    let mut trans = vec![];
    for i in 0..len {
        let mut temp = vec![];
        for v in &vec {
            if v.len() != len {
                return Err(vm.error_index("Element size differs."));
            }
            temp.push(v[i]);
        }
        let ary = Value::array_from(temp);
        trans.push(ary);
    }
    //aref.elements.reverse();
    let res = Value::array_from(trans);
    Ok(res)
}

fn min(vm: &mut VM, self_val: Value, _args: &Args) -> VMResult {
    fn to_float(vm: &VM, val: Value) -> Result<f64, RubyError> {
        if val.is_packed_fixnum() {
            Ok(val.as_packed_fixnum() as f64)
        } else if val.is_packed_num() {
            Ok(val.as_packed_flonum())
        } else {
            Err(vm.error_type("Currently, each element must be Numeric."))
        }
    }

    let aref = self_val.as_array().unwrap();
    if aref.elements.len() == 0 {
        return Ok(Value::nil());
    }
    let mut min_obj = aref.elements[0];
    let mut min = to_float(vm, min_obj)?;
    for elem in &aref.elements {
        let elem_f = to_float(vm, *elem)?;
        if elem_f < min {
            min_obj = *elem;
            min = elem_f;
        }
    }

    return Ok(min_obj);
}

fn max(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    if aref.elements.len() == 0 {
        return Ok(Value::nil());
    }
    let mut max = aref.elements[0];
    for elem in &aref.elements {
        if vm.eval_gt(max, *elem)? {
            max = *elem;
        };
    }
    Ok(max)
}

fn fill(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 1)?;
    let aref = self_val.as_mut_array().unwrap();
    for elem in &mut aref.elements {
        *elem = args[0];
    }
    Ok(self_val)
}

fn clear(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_mut_array().unwrap();
    aref.elements.clear();
    Ok(self_val)
}

fn uniq_(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;

    let mut set = std::collections::HashSet::new();
    match args.block {
        None => {
            let aref = self_val.as_mut_array().unwrap();
            aref.elements.retain(|x| set.insert(HashKey(*x)));
            Ok(self_val)
        }
        Some(block) => {
            let aref = self_val.as_mut_array().unwrap();
            let mut block_args = Args::new1(Value::nil());
            aref.elements.retain(|x| {
                block_args[0] = *x;
                let res = vm.eval_block(block, &block_args).unwrap();
                vm.temp_push(res);
                set.insert(HashKey(res))
            });
            Ok(self_val)
        }
    }
}

fn slice_(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 2)?;
    let start = args[0].expect_integer(vm, "Currently, first arg must be Integer.")?;
    if start < 0 {
        return Err(vm.error_argument("First arg must be positive value."));
    };
    let len = args[1].expect_integer(vm, "Currently, second arg must be Integer")?;
    if len < 0 {
        return Err(vm.error_argument("Second arg must be positive value."));
    };
    let start = start as usize;
    let len = len as usize;
    let aref = self_val.as_mut_array().unwrap();
    let new = aref.elements.drain(start..start + len).collect();
    Ok(Value::array_from(new))
}

fn first(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    if aref.elements.len() == 0 {
        Ok(Value::nil())
    } else {
        Ok(aref.elements[0])
    }
}

fn last(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    if aref.elements.len() == 0 {
        Ok(Value::nil())
    } else {
        Ok(*aref.elements.last().unwrap())
    }
}

fn dup(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    Ok(Value::array_from(aref.elements.clone()))
}

fn pack(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_range(args.len(), 0, 1)?;
    let aref = self_val.as_array().unwrap();
    let mut v = vec![];
    for elem in &aref.elements {
        let i = match elem.as_integer() {
            Some(i) => i as i8 as u8,
            None => return Err(vm.error_argument("Must be Array of Integer.")),
        };
        v.push(i);
    }
    Ok(Value::bytes(v))
}

fn join(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_range(args.len(), 0, 1)?;
    let sep = if args.len() == 0 {
        ""
    } else {
        match args[0].as_string() {
            Some(s) => s,
            None => return Err(vm.error_argument("Seperator must be String.")),
        }
    };
    let aref = self_val.as_array().unwrap();
    let mut res = "".to_string();
    for elem in &aref.elements {
        let s = vm.val_to_s(*elem);
        if res.is_empty() {
            res = s.to_owned();
        } else {
            res = res + sep + s.as_str();
        }
    }
    Ok(Value::string(res))
}

fn drop(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 1)?;
    let aref = self_val.as_array().unwrap();
    let num = args[0].expect_integer(vm, "An argument must be Integer.")? as usize;
    let ary = &aref.elements[num..aref.elements.len()];
    Ok(Value::array_from(ary.to_vec()))
}

fn zip(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    let self_ary = self_val.as_array().unwrap();
    let mut args_ary = vec![];
    for a in args.iter() {
        args_ary.push(a.clone().expect_array(vm, "Args")?.elements.clone());
    }
    let mut ary = vec![];
    for (i, val) in self_ary.elements.iter().enumerate() {
        let mut vec = vec![*val];
        for args in &args_ary {
            if i < args.len() {
                vec.push(args[i]);
            } else {
                vec.push(Value::nil());
            }
        }
        let zip = Value::array_from(vec);
        ary.push(zip);
    }
    match args.block {
        Some(block) => {
            let mut arg = Args::new1(Value::nil());
            vm.temp_push_vec(&mut ary.clone());
            for val in ary {
                arg[0] = val;
                vm.eval_block(block, &arg)?;
            }
            Ok(Value::nil())
        }
        None => Ok(Value::array_from(ary)),
    }
}

fn grep(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 1)?;
    let aref = self_val.as_array().unwrap();
    let ary = match args.block {
        None => aref
            .elements
            .iter()
            .filter_map(|x| match vm.eval_teq(*x, args[0]) {
                Ok(true) => Some(*x),
                _ => None,
            })
            .collect(),
        Some(_block) => vec![],
    };
    Ok(Value::array_from(ary))
}

fn sort(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    //use std::cmp::Ordering;
    vm.check_args_num(self_val, args.len(), 0)?;
    let mut ary = self_val.expect_array(vm, "Receiver")?.elements.clone();
    match args.block {
        None => {
            vm.sort_array(&mut ary)?;
        }
        Some(_block) => return Err(vm.error_argument("Currently, can not use block.")),
    };
    Ok(Value::array_from(ary))
}

use std::collections::HashSet;
fn uniq(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(self_val, args.len(), 0)?;
    let aref = self_val.as_array().unwrap();
    let mut h: HashSet<HashKey> = HashSet::new();
    let mut v = vec![];
    match args.block {
        None => {
            for elem in &aref.elements {
                if h.insert(HashKey(*elem)) {
                    v.push(*elem);
                };
            }
        }
        Some(_block) => return Err(vm.error_argument("Currently, can not use block.")),
    };
    Ok(Value::array_from(v))
}

fn any_(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    let aref = self_val.as_array().unwrap();
    if args.len() == 1 {
        if args.block.is_some() {
            eprintln!("warning: given block not used");
        }
        for v in aref.elements.iter() {
            if vm.eval_teq(*v, args[0])? {
                return Ok(Value::true_val());
            };
        }
        return Ok(Value::false_val());
    }
    vm.check_args_num(self_val, args.len(), 0)?;

    if let Some(method) = args.block {
        let mut args = Args::new1(Value::nil());
        for v in aref.elements.iter() {
            args[0] = *v;
            if vm.eval_block(method, &args)?.to_bool() {
                return Ok(Value::true_val());
            };
        }
    } else {
        for v in aref.elements.iter() {
            if v.to_bool() {
                return Ok(Value::true_val());
            };
        }
    }
    Ok(Value::false_val())
}

fn all_(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    let aref = self_val.as_array().unwrap();
    if args.len() == 1 {
        if args.block.is_some() {
            eprintln!("warning: given block not used");
        }
        for v in aref.elements.iter() {
            if !vm.eval_teq(*v, args[0])? {
                return Ok(Value::false_val());
            };
        }
        return Ok(Value::true_val());
    }
    vm.check_args_num(self_val, args.len(), 0)?;

    if let Some(method) = args.block {
        let mut args = Args::new1(Value::nil());
        for v in aref.elements.iter() {
            args[0] = *v;
            if !vm.eval_block(method, &args)?.to_bool() {
                return Ok(Value::false_val());
            };
        }
    } else {
        for v in aref.elements.iter() {
            if !v.to_bool() {
                return Ok(Value::false_val());
            };
        }
    }
    Ok(Value::true_val())
}

fn count(vm: &mut VM, mut self_val: Value, args: &Args) -> VMResult {
    vm.check_args_range(args.len(), 0, 1)?;
    if args.block.is_some() {
        return Err(vm.error_argument("Currently, block is not supported."));
    }
    let ary = self_val.expect_array(vm, "").unwrap();
    match args.len() {
        0 => {
            let len = ary.len() as i64;
            Ok(Value::integer(len))
        }
        1 => {
            let other = args[0];
            let mut count = 0;
            for elem in &ary.elements {
                if vm.eval_eq(*elem, other) {
                    count += 1;
                }
            }
            Ok(Value::integer(count))
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[test]
    fn array() {
        let program = "
        a=[1,2,3,4]
        assert 3, a[2];
        a[1]=14
        assert [1,14,3,4], a
        a.pop()
        assert [1,14,3], a
        a.push(7,8,9)
        assert [1,14,3,7,8,9], a
        a=[1,2,3,4]
        b=Array.new(a)
        assert a, b
        b[2]=100
        assert [1,2,3,4], a
        assert [1,2,100,4], b
        assert 4, a.length
        assert 4, b.length
        assert true, [].empty?
        assert false, a.empty?
        a = [1,2,3]
        b = [4,5]
        assert [1,2,3,4,5], a.concat(b)
        assert [1,2,3,4,5], a
        assert [4,5], b
    ";
        assert_script(program);
    }

    #[test]
    fn array1() {
        let program = "
        assert [], [1,2,3]*0 
        assert [1,2,3]*1, [1,2,3]
        assert [nil,nil,nil,nil,nil], [nil]*5 
        assert [1,2,3,3,4,5], [1,2,3]+[3,4,5] 
        assert [1,2], [1,2,3]-[3,4,5] 
    ";
        assert_script(program);
    }

    #[test]
    fn array2() {
        let program = "
        a = [1,2,3,4,5,6,7]
        b = [3,9]
        c = [3,3]
        assert 3, a[2]
        assert [4,5,6,7], a[3,9]
        assert [4,5,6,7], a[*b] 
        assert [4,5,6], a[3,3]
        assert [4,5,6], a[*c] 
        assert nil, a[7]
        assert [], a[7,3]
    ";
        assert_script(program);
    }

    #[test]
    fn array3() {
        let program = "
        a = [1,2,3,4,5,6,7]
        assert [3,4,5], a[2,3]
        a[2,3] = 100
        assert [1,2,100,6,7], a
        ";
        assert_script(program);
    }

    #[test]
    fn array_shift() {
        // TODO: 'a.unshift [0]' is parsed as 'a.unshift[0]'. This is wrong.
        let program = "
        a = [0, 1, 2, 3, 4]
        assert 0, a.shift
        assert [1, 2, 3, 4], a
        assert [1], a.shift(1)
        assert [2,3], a.shift(2)
        assert [4], a
        assert nil, [].shift
        assert [],  [].shift(1)

        a = [1,2,3]
        a.unshift 0
        assert [0, 1, 2, 3], a
        a.unshift([0])
        assert [[0], 0, 1, 2, 3], a
        a.unshift 1, 2
        assert [1, 2, [0], 0, 1, 2, 3], a
        ";
        assert_script(program);
    }

    #[test]
    fn array_cmp() {
        let program = "
        assert(0, [1,2,3,4] <=> [1,2,3,4])
        assert(1, [1,2,3,4] <=> [1,2,3])
        assert(-1, [1,2,3,4] <=> [1,2,3,4,5])
        assert(1, [1,2,3,4] <=> [-1,2,3,4,5,6])
        assert(-1, [1,2,3,4] <=> [6,2])
        assert(nil, [1,2,3,4] <=> 8)
        ";
        assert_script(program);
    }

    #[test]
    fn array_mul() {
        let program = r#"
        assert [1,2,3,1,2,3,1,2,3], [1,2,3] * 3 
        assert "1,2,3", [1,2,3] * ","
        assert "Ruby", ["Ruby"] * ","
        a = ["Ruby"] * 5
        assert ["Ruby", "Ruby", "Ruby", "Ruby", "Ruby"], a
        a[2] = "mruby"
        assert ["Ruby", "Ruby", "mruby", "Ruby", "Ruby"], a
        "#;
        assert_script(program);
    }

    #[test]
    fn array_push() {
        let program = r#"
        a = [1,2,3]
        a << 4
        a << "Ruby"
        assert([1,2,3,4,"Ruby"], a)
        "#;
        assert_script(program);
    }

    #[test]
    fn array_sort() {
        let program = r#"
        assert([-3,2,6], [6,2,-3].sort)
        assert([-3,2.34,6.3], [6.3,2.34,-3].sort)
        assert_error {[1,2.5,nil].sort}
        "#;
        assert_script(program);
    }

    #[test]
    fn array_min_max() {
        let program = r#"
        assert nil, [].min
        #assert [], [].min(1)
        assert 2, [2, 5, 3.7].min
        #assert [2, 3], [2, 5, 3].min(2)
        assert nil, [].max
        #assert [], [].max(1)
        assert 5, [2, 5, 3].max
        #assert [5, 3], [2.1, 5, 3].max(2)
        "#;
        assert_script(program);
    }

    #[test]
    fn array_map() {
        let program = "
        a = [1,2,3]
        assert(a.map {|| 3 }, [3,3,3])
        assert(a.map {|x| x*3 }, [3,6,9])
        assert(a.map do |x| x*3 end, [3,6,9])
        assert(a, [1,2,3])
        b = [1, [2, 3], 4, [5, 6, 7]]
        assert [2, 2, 3, 2, 3, 8, 5, 6, 7, 5, 6, 7], b.flat_map{|x| x * 2}
        ";
        assert_script(program);
    }

    #[test]
    fn array_each() {
        let program = "
        a = [1,2,3]
        b = 0
        assert([1,2,3], a.each {|x| b+=x })
        assert(6, b)
    ";
        assert_script(program);
    }

    #[test]
    fn array_include() {
        let program = r#"
        a = ["ruby","rust","java"]
        assert(true, a.include?("ruby"))
        assert(true, a.include?("rust"))
        assert(false, a.include?("c++"))
        assert(false, a.include?(:ruby))
    "#;
        assert_script(program);
    }

    #[test]
    fn array_reverse() {
        let program = "
    a = [1,2,3,4,5]
    assert([5,4,3,2,1], a.reverse)
    assert([1,2,3,4,5], a)
    assert([5,4,3,2,1], a.reverse!)
    assert([5,4,3,2,1], a)
    ";
        assert_script(program);
    }

    #[test]
    fn array_rotate() {
        let program = r#"
        a = ["a","b","c","d"]
        assert ["b","c","d","a"], a.rotate!
        assert ["b","c","d","a"], a
        assert ["d","a","b","c"], a.rotate!(2)
        assert ["a","b","c","d"], a.rotate!(-3) 
    "#;
        assert_script(program);
    }

    #[test]
    fn array_transpose() {
        let program = r#"
        assert [[1, 3, 5], [2, 4, 6]], [[1,2],[3,4],[5,6]].transpose
        assert [], [].transpose

        assert_error { [1,2,3].transpose }
        assert_error { [[1,2],[3,4,5],[6,7]].transpose }
        "#;
        assert_script(program);
    }

    #[test]
    fn array_zip() {
        let program = r#"
        assert [[1,4,7],[2,5,8],[3,6,9]], [1,2,3].zip([4,5,6],[7,8,9])
        assert [[1,:a,:A],[2,:b,:B]], [1,2].zip([:a,:b,:c],[:A,:B,:C,:D])
        assert [[1,:a,:A],[2,:b,:B],[3,:c,:C],[4,nil,:D],[5,nil,nil]], [1,2,3,4,5].zip([:a,:b,:c],[:A,:B,:C,:D])
        ans = []
        [1,2,3].zip([4,5,6], [7,8,9]) {|ary|
            ans.push(ary)
        }
        assert [[1,4,7],[2,5,8],[3,6,9]], ans
        "#;
        assert_script(program);
    }

    #[test]
    fn fill() {
        let program = r#"
        a = [1,2,3,4]
        assert ["Ruby","Ruby","Ruby","Ruby"], a.fill("Ruby")
        assert ["Ruby","Ruby","Ruby","Ruby"], a
        "#;
        assert_script(program);
    }

    #[test]
    fn array_methods() {
        let program = r#"
        a = [1,2,3,4]
        assert [], a.clear
        assert [], a
        assert 1, [1,2,3,4].first
        assert 4, [1,2,3,4].last
        assert nil, [].first
        assert nil, [].last
        a = ["a","b","c"]
        assert ["b","c"], a.slice!(1, 2)
        assert ["a"], a
        a = ["a","b","c"]
        assert [], a.slice!(1, 0)
        assert ["a","b","c"], a
        "#;
        assert_script(program);
    }

    #[test]
    fn uniq() {
        let program = r#"
        a = [1,2,3,4,3,2,1,0,3.0]
        assert [1,2,3,4,0,3.0], a.uniq
        assert [1,2,3,4,3,2,1,0,3.0], a

        assert [1,2,3,3.0], a.uniq! {|x| x % 3 }
        "#;
        assert_script(program);
    }

    #[test]
    fn count() {
        let program = r#"
        a = [1, "1", 1.0, :one, 1, :one, 53.7, [1]]
        assert 8, a.count
        assert 3, a.count(1)
        assert 2, a.count(:one)
        "#;
        assert_script(program);
    }

    #[test]
    fn any_() {
        let program = r#"
        assert false, [1, 2, 3].any? {|v| v > 3 }
        assert true, [1, 2, 3].any? {|v| v > 1 }
        assert false, [].any? {|v| v > 0 }
        assert false, %w[ant bear cat].any?(/d/)
        assert true, %w[ant bear cat].any?(/ear/)
        assert true, [nil, true, 99].any?(Integer)
        assert false, [nil, true, 99].any?(String)
        assert false, [nil, false, nil].any?
        assert true, [nil, true, 99].any?
        assert false, [].any?
        "#;
        assert_script(program);
    }

    #[test]
    fn all_() {
        let program = r#"
        assert true, [5, 6, 7].all? {|v| v > 0 }
        assert false, [5, -1, 7].all? {|v| v > 0 }
        assert true, [].all? {|v| v > 0 }
        assert true, [-50, 77, 99].all?(Integer)
        assert false, [3, 111, :d].all?(Integer)
        assert false, %w[ant bear cat].all?(/t/)
        assert true, %w[ant bear cat].all?(/a/)
        "#;
        assert_script(program);
    }
}
