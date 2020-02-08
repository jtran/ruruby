use crate::util::*;
use crate::vm::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: Option<IdentId>,
    pub method_table: MethodTable,
    pub superclass: PackedValue,
    pub is_singleton: bool,
}

impl ClassInfo {
    pub fn new(name: impl Into<Option<IdentId>>, superclass: PackedValue) -> Self {
        ClassInfo {
            name: name.into(),
            method_table: HashMap::new(),
            superclass,
            is_singleton: false,
        }
    }
}

pub type ClassRef = Ref<ClassInfo>;

impl ClassRef {
    /*
    pub fn from_no_superclass(id: impl Into<Option<IdentId>>) -> Self {
        ClassRef::new(ClassInfo::new(id, PackedValue::nil()))
    }*/

    pub fn from(
        id: impl Into<Option<IdentId>>,
        superclass: impl Into<Option<PackedValue>>,
    ) -> Self {
        let superclass = match superclass.into() {
            Some(superclass) => superclass,
            None => PackedValue::nil(),
        };
        ClassRef::new(ClassInfo::new(id, superclass))
    }

    pub fn superclass(&self) -> Option<ClassRef> {
        if self.superclass.is_nil() {
            None
        } else {
            Some(self.superclass.as_class())
        }
    }
}

pub fn init_class(globals: &mut Globals) {
    let class = globals.class_class;
    globals.add_builtin_instance_method(class, "new", class_new);
    globals.add_builtin_instance_method(class, "superclass", superclass);
    globals.add_builtin_class_method(globals.class, "new", class_class_new);
}

/// Built-in function "new".
fn class_class_new(vm: &mut VM, _args: &Args, _block: Option<MethodRef>) -> VMResult {
    let id = vm.globals.get_ident_id("nil");
    let classref = ClassRef::from(id, vm.globals.object);
    let val = PackedValue::class(&mut vm.globals, classref);

    Ok(val)
}

/// Built-in function "new".
fn class_new(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
    let new_instance = PackedValue::ordinary_object(args.self_value);
    // call initialize method.
    if let Some(methodref) = args.self_value.get_instance_method(IdentId::INITIALIZE) {
        let iseq = vm.globals.get_method_info(methodref).as_iseq(&vm)?;
        let mut args = args.clone();
        args.self_value = new_instance;
        vm.vm_run(iseq, None, &args, None, None)?;
        vm.stack_pop();
    };
    Ok(new_instance)
}

fn superclass(vm: &mut VM, args: &Args, _block: Option<MethodRef>) -> VMResult {
    let class = vm.val_as_class(args.self_value)?;
    Ok(class.superclass)
}
