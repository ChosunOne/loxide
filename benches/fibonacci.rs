use std::io::{stderr, stdout};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use loxide::vm::VM;

pub fn fibonacci_benchmark(c: &mut Criterion) {
    let source = r#"
        fun fib(x) {
            if (x < 2) {
                return 1;
            }
            return fib(x - 1) + fib(x - 2);
        }
        fib(20);
    "#;
    let mut vm = VM::new(stdout(), stderr());
    c.bench_function("fib 20", |b| b.iter(|| vm.interpret(black_box(source))));
}

criterion_group!(benches, fibonacci_benchmark);
criterion_main!(benches);
