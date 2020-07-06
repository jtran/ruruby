#[cfg(feature = "perf")]
use crate::vm::perf::*;
use crate::*;
use std::path::PathBuf;

pub fn eval_script(script: impl Into<String>, expected: Value) {
    let mut globals = GlobalsRef::new_globals();
    let mut vm = globals.new_vm();
    let res = vm.run(PathBuf::from(""), &script.into(), None);
    #[cfg(feature = "perf")]
    {
        let mut perf = Perf::new();
        let globals = vm.globals;
        for vm in &globals.fibers {
            perf.add(&vm.perf);
        }
        perf.print_perf();
    }
    #[cfg(feature = "gc-debug")]
    {
        ALLOC.with(|a| a.borrow_mut().as_ref().unwrap().print_mark());
    }
    match res {
        Ok(res) => {
            if res != expected {
                panic!("Expected:{:?} Got:{:?}", expected, res);
            }
        }
        Err(err) => {
            err.show_err();
            err.show_loc(0);
            panic!("Got error: {:?}", err);
        }
    }
}

pub fn assert_script(script: impl Into<String>) {
    let mut globals = GlobalsRef::new_globals();
    let mut vm = globals.new_vm();
    let res = vm.run(PathBuf::from(""), &script.into(), None);
    #[cfg(feature = "perf")]
    {
        let mut perf = Perf::new();
        let globals = vm.globals;
        for vm in &globals.fibers {
            perf.add(&vm.perf);
        }
        perf.print_perf();
    }
    #[cfg(feature = "gc-debug")]
    {
        ALLOC.with(|a| a.borrow_mut().as_ref().unwrap().print_mark());
    }
    match res {
        Ok(_) => {}
        Err(err) => {
            err.show_err();
            err.show_loc(0);
            panic!("Got error: {:?}", err);
        }
    }
}

pub fn assert_error(script: impl Into<String>) {
    let mut globals = GlobalsRef::new_globals();
    let mut vm = globals.new_vm();
    let program = script.into();
    let res = vm.run(PathBuf::from(""), &program, None);
    #[cfg(feature = "perf")]
    {
        let mut perf = Perf::new();
        let globals = vm.globals;
        for vm in &globals.fibers {
            perf.add(&vm.perf);
        }
        perf.print_perf();
    }
    #[cfg(feature = "gc-debug")]
    {
        ALLOC.with(|a| a.borrow_mut().as_ref().unwrap().print_mark());
    }
    match res {
        Ok(_) => panic!("Must be an error:{}", program),
        Err(err) => {
            err.show_err();
            err.show_loc(0);
        }
    }
}
