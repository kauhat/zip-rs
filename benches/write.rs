use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main};
use criterion::{BenchmarkGroup, BenchmarkId, Criterion};

use std::io::{Cursor, Read, Write};
use std::ops::Add;
use std::str::from_utf8_mut;

use seeder::{ArchiveBuilder, ArchiveFaker, LoremSeeder, RandomSeeder, Seeder};
use zip::write::FileOptions;
use zip::SUPPORTED_COMPRESSION_METHODS;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

fn bench_write_whole_all_methods<T>(mut group: BenchmarkGroup<T>, builder: &ArchiveBuilder)
where
    T: Measurement,
{
    let size = 1024; // * 1024;

    //
    for &method in SUPPORTED_COMPRESSION_METHODS {
        let mut in_buffer = Vec::with_capacity(size);

        &group.bench_with_input(
            BenchmarkId::from_parameter(method),
            &method,
            |bench, method| {
                in_buffer.clear();
                builder.reset();

                bench.iter(|| {
                    builder
                        .file_options_from(|| FileOptions::default().compression_method(*method))
                        .generate(in_buffer);
                });
            },
        );
    }

    &group.finish();
}

fn bench_write_lorem(bench: &mut Criterion) {
    let size = 1024 * 1024;

    // Generate a test archive...
    let file = &mut Cursor::new(Vec::new());
    let builder = ArchiveFaker::fake_archive()
        .file_names_from(|i: i32| -> i32 { i + 1 })
        .file_data_from(RandomSeeder::fill);

    bench_write_whole_all_methods(bench.benchmark_group("write_lorem_archive"), builder);
}

fn bench_write_random(bench: &mut Criterion) {
    let size = 1024 * 1024;

    // Generate a test archive...
    let file = &mut Cursor::new(Vec::new());
    let builder = ArchiveFaker::fake_archive()
        .file_names_from(|i: i32| -> i32 { i + 1 })
        .file_data_from(RandomSeeder::fill);

    //
    bench_write_whole_all_methods(bench.benchmark_group("read_random_archive"), builder);
}

criterion_group!(read_archive, bench_write_lorem, bench_write_random);
criterion_main!(read_archive);
