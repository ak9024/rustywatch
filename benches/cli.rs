use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;

fn bench_cli(c: &mut Criterion) {
    c.bench_function("run CLI command", |b| {
        b.iter(|| {
            let cmd = Command::new("cargo")
                .arg("run")
                .output()
                .expect("Failed to execute command");

            black_box(cmd.stdout);
        })
    });
}

criterion_group!(benches, bench_cli);
criterion_main!(benches);
