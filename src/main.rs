use std::io::Write;

use rlox::vm::{InterpretResult, Vm};
compile_error!("https://craftinginterpreters.com/global-variables.html#error-synchronization");
fn main() {
    let mut vm = Vm::init();
    if std::env::args().len() > 2 {
        eprintln!("Usage: rlox [path]");
        std::process::exit(64);
    }
    if let Some(file) = std::env::args().nth(1) {
        let src = std::fs::read_to_string(file).unwrap();
        match vm.interpret(src) {
            InterpretResult::CompileError => std::process::exit(64),
            InterpretResult::RuntimeError => std::process::exit(70),
            InterpretResult::Ok => {}
        }
    } else {
        repl(vm)
    }
}

fn repl(mut vm: Vm) {
    loop {
        let mut cmd = String::with_capacity(1024);
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut cmd).unwrap();
        vm.interpret(cmd);
    }
}
