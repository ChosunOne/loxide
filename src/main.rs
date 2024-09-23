use loxide::{error::Error, vm::VM};
use std::{
    env, fs,
    io::{stderr, stdin, stdout, Write},
};

fn repl<Out: Write, EOut: Write>(mut vm: VM<Out, EOut>) {
    loop {
        let mut line = String::new();
        print!("> ");
        let _ = stdout().flush();
        stdin().read_line(&mut line).expect("Malformed input.");
        if let Err(e) = vm.interpret(&line) {
            eprintln!("{e}")
        }
    }
}

fn run_file<Out: Write, EOut: Write>(path: &str, mut vm: VM<Out, EOut>) -> Result<(), Error> {
    let source = fs::read_to_string(path).expect("Failed to read file.");
    vm.interpret(&source)
}

fn main() -> Result<(), Error> {
    let vm = VM::new(stdout(), stderr());
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        repl(vm);
    } else if args.len() == 2 {
        run_file(&args[1], vm)?;
    } else {
        eprintln!("Usage: loxide [path]");
    }

    Ok(())
}
