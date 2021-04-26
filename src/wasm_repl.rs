use std::sync::Mutex;
use std::lazy::SyncLazy;
use crate::*;
use wasm_bindgen::prelude::*;

static GLOBAL_REPL: SyncLazy<Mutex<Option<Repl>>> = SyncLazy::new(|| {
    Mutex::new(None)
});

pub struct Repl {
    program: String,
    parser: Parser,
    vm: VMRef,
    context: ContextRef,
}

#[wasm_bindgen]
pub fn new_repl() {
    let program = String::new();
    let parser = Parser::new();
    let mut globals = GlobalsRef::new_globals();
    let mut vm = globals.create_main_fiber();
    vm.set_global_var(IdentId::get_id("$0"), Value::string("irb"));
    let context = ContextRef::new_heap(
        vm.globals.main_object,
        Block::None,
        ISeqRef::default(),
        None,
    );

    let repl = Repl {
        program,
        parser,
        vm,
        context,
    };
    let mut repl_opt = GLOBAL_REPL.lock().expect("failed to acquire lock");
    repl_opt.replace(repl);
}

#[wasm_bindgen]
pub fn run_repl_line(line: &str) {
    let mut repl_opt = GLOBAL_REPL.lock().expect("failed to acquire lock");
    let mut repl = repl_opt.as_mut().expect("you need to call new_repl() first to initialize it");
    let mut line = line.to_string();
    line.push('\n');

    repl.program = format!("{}{}\n", repl.program, line);

    match repl.parser.clone().parse_program_repl(
        std::path::PathBuf::from("REPL"),
        &repl.program,
        Some(repl.context),
    ) {
        Ok(parse_result) => {
            let source_info = parse_result.source_info;
            match repl.vm.run_repl(parse_result, repl.context) {
                Ok(result) => {
                    repl.parser.lexer.source_info = source_info;
                    println!("=> {:?}", result);
                }
                Err(err) => {
                    for (info, loc) in &err.info {
                        info.show_loc(loc);
                    }
                    err.show_err();
                    repl.vm.clear();
                }
            }
            repl.program = String::new();
        }
        Err(err) => {
            if RubyErrorKind::ParseErr(ParseErrKind::UnexpectedEOF) == err.kind {
                return;
            }
            err.show_loc(0);
            err.show_err();
            repl.program = String::new();
        }
    }
}
