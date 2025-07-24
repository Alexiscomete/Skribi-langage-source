use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use skribi_language_source::{execute::{Execute, ExecutionContext}, get_file_content::get_content, parse::parse, tokens::tokenize, execute};

macro_rules! create_general {
    ($c: expr, $name: expr, $path: expr) => {
        $c.bench_function($name, |b| {
            b.iter(|| execute(black_box(vec!["".to_owned(), $path.to_owned()]), black_box(false)))
        });
    };
}

macro_rules! create_execute {
    ($c: expr, $name: expr, $path: expr) => {
        $c.bench_function($name, |b| {
            b.iter_batched(
                || {
                    let extension: Vec<String> = vec!["skrb".to_string()];
                    let args = vec!["".to_owned(), $path.to_owned()];
                    let content = get_content(args, extension.clone()).unwrap();
                    let tokens = tokenize(content).unwrap();
                    black_box(parse(tokens).unwrap().unwrap())
                },
                |data| {
                    black_box(data).execute(&mut black_box(ExecutionContext::new()))
                }, criterion::BatchSize::PerIteration);
        });
    };
}

macro_rules! create_function {
    ($c: expr, $path: expr) => {
        create_general!(
            $c,
            concat!($path, " 01 - ALL"),
            $path
        );
        create_execute!($c, concat!($path, " 02 - EXECUTE"), $path)
    };
}

pub fn fibo_benchmark(c: &mut Criterion) {
    create_function!(
        c,
        "resources/test_programs/algo/fibo.skrb"
    );
}

pub fn or_eq_benchmark(c: &mut Criterion) {
    create_function!(c, "resources/test_programs/cmp/or_eq.skrb");
}

criterion_group!(algo, fibo_benchmark, or_eq_benchmark);
criterion_main!(algo);
