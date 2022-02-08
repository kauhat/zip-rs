use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main};
use criterion::{BenchmarkGroup, BenchmarkId, Criterion};

use std::io::{Cursor, Read, Write};
use std::ops::Add;
use std::str::from_utf8_mut;

use fake::faker::lorem::en::*;
use fake::{Dummy, Fake, Faker};
use getrandom::getrandom;
use seeder::{ArchiveBuilder, ArchiveFaker, LoremSeeder, RandomSeeder, Seeder};
use zip::write::FileOptions;
use zip::SUPPORTED_COMPRESSION_METHODS;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

fn generate_whole_archive(
    in_buffer: &mut Vec<u8>,
    size: usize,
    seeder: &dyn Seeder,
    base_options: &FileOptions,
) {
    let mut writer = ZipWriter::new(Cursor::new(in_buffer));
    let options = base_options.clone();

    writer.start_file("random.dat", options).unwrap();

    // Generate some random data.
    let mut bytes = vec![0u8; size];
    seeder.fill(bytes.as_mut_slice());

    writer.write_all(&bytes).unwrap();

    writer.finish();
}

fn read_whole_archive(in_buffer: &[u8]) -> usize {
    let mut archive = ZipArchive::new(Cursor::new(in_buffer)).unwrap();
    let mut file = archive.by_name("random.dat").unwrap();
    let mut buf = [0u8; 1024];

    let mut total_bytes = 0;

    loop {
        let n = file.read(&mut buf).unwrap();
        total_bytes += n;
        if n == 0 {
            return total_bytes;
        }
    }
}

fn bench_read_whole_all_methods<T>(mut group: BenchmarkGroup<T>, builder: &ArchiveBuilder)
where
    T: Measurement,
{
    let size = 1024 * 1024;

    for &method in SUPPORTED_COMPRESSION_METHODS {
        // Write a test versions of the archive for each compression method

        let mut in_buffer = Vec::with_capacity(size);
        builder
            .file_options_from(|| FileOptions::default().compression_method(*method))
            .generate(in_buffer);

        &group.bench_with_input(
            BenchmarkId::from_parameter(method),
            method,
            |bench, method| {
                builder.reset();

                bench.iter(|| {
                    let size = read_whole_archive(&in_buffer.as_slice());
                });
            },
        );
    }

    &group.finish();
}

fn bench_read_lorem(bench: &mut Criterion) {
    let size = 1024 * 1024;

    // Generate a source of truth archive...
    let file = &mut Cursor::new(Vec::new());
    ArchiveFaker::fake_archive()
        .file_names_from(|i: i32| -> i32 { i + 1 })
        .file_data_from(LoremSeeder::fill)
        .build(file);

    // Read the test archive with all methods.
    bench_read_whole_all_methods(bench.benchmark_group("read_lorem_archive"), file);
}

fn bench_read_random(bench: &mut Criterion) {
    let size = 1024 * 1024;

    // Generate a test archive...
    let file = &mut Cursor::new(Vec::new());
    ArchiveFaker::fake_archive()
        .file_names_from(|i: i32| -> i32 { i + 1 })
        .file_data_from(RandomSeeder::fill)
        .build(file);

    // Write test versions of the archive for each compression method

    // Read the test archive with all methods.
    bench_read_whole_all_methods(bench.benchmark_group("read_random_archive"), file);
}

criterion_group!(benches, bench_read_lorem, bench_read_random);
criterion_main!(benches);
