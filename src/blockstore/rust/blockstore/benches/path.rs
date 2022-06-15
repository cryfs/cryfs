use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::{Path, PathBuf};

use cryfs_blockstore::utils::path::path_join;

fn bench_join(c: &mut Criterion) {
    let p1 = black_box(PathBuf::from("/absolute/path/with//double//slash"));
    let p2 = black_box(Path::new("/some/path/with//double//slash"));
    let p3 = black_box(Path::new("relative/path/with//double//slash"));

    c.bench_function(
        "path_join",
        |b| {
            b.iter(|| black_box(path_join(&[p1.as_path(), p2, p3])));
        },
    );

    c.bench_function(
        "PathBuf::join",
        |b| {
            b.iter(|| black_box(p1.join(p2).join(p3)));
        },
    );

    c.bench_function(
        "PathBuf::extend",
        |b| {
            b.iter(|| black_box(p1.clone().extend(&[p2, p3])));
        },
    );

    c.bench_function(
        "PathBuf::push",
        |b| {
            b.iter(|| black_box({let mut r = p1.clone(); r.push(p2); r.push(p3);}));
        },
    );
}

criterion_group!(benches, bench_join);
criterion_main!(benches);
