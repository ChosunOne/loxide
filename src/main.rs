use loxide::vm::VM;
use std::{
    env,
    io::{stdin, stdout, Write},
};

fn repl(mut vm: VM) {
    loop {
        let mut line = String::new();
        print!("> ");
        let _ = stdout().flush();
        stdin().read_line(&mut line).expect("Malformed input.");
        vm.interpret(&line);
    }
}

fn run_file(path: &str, mut vm: VM) {
    todo!();
}

fn main() {
    let vm = VM::new();
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        repl(vm);
    } else if args.len() == 2 {
        run_file(&args[1], vm);
    } else {
        eprintln!("Usage: loxide [path]");
    }
}
