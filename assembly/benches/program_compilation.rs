use criterion::{criterion_group, criterion_main, Criterion};
use miden_assembly::{self, Assembler};
use std::time::Duration;

fn program_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("program_compilation");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("sha256", |bench| {
        let source = "
            use.std::crypto::hashes::sha256

            begin
                exec.sha256::hash
            end";
        bench.iter(|| {
            let assembler = Assembler::default();
            assembler
                .compile(source)
                .expect("Failed to compile test source.")
        });
    });

    group.finish();
}

criterion_group!(sha256_group, program_compilation);
criterion_main!(sha256_group);
