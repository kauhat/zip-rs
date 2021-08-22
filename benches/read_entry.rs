use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main};
use criterion::{BenchmarkGroup, BenchmarkId, Criterion};

use std::io::{Cursor, Read, Write};

use rand::Rng;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

trait Seeder {
    // TODO: ...
    // pub fn fill<T>(&self, dest:&mut T);
}

struct RandomSeeder {}
impl Seeder for RandomSeeder {}

struct LoremSeeder {}
impl Seeder for LoremSeeder {}

fn generate_whole_archive(
    buffer: &mut Vec<u8>,
    size: usize,
    seeder: &dyn Seeder,
    base_options: &FileOptions,
) {
    let mut writer = ZipWriter::new(Cursor::new(buffer));
    let options = base_options.clone();

    writer.start_file("random.dat", options).unwrap();

    // Generate some random data.
    let mut bytes = vec![0u8; size];
    rand::thread_rng().fill(bytes.as_mut_slice());

    writer.write_all(&bytes).unwrap();

    writer.finish();
}

fn read_whole_archive(buffer: &[u8]) -> usize {
    let mut archive = ZipArchive::new(Cursor::new(buffer)).unwrap();
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

fn bench_write_whole_all_methods<T>(mut group: BenchmarkGroup<T>, seeder: &dyn Seeder)
where
    T: Measurement,
{
    let size = 1024 * 1024;

    //
    for method in CompressionMethod::supported_methods().iter() {
        let mut buffer = Vec::with_capacity(size);

        &group.bench_with_input(
            BenchmarkId::from_parameter(method),
            method,
            |bench, method| {
                buffer.clear();

                bench.iter(|| {
                    let mut bytes = generate_whole_archive(
                        &mut buffer,
                        size,
                        seeder,
                        &FileOptions::default().compression_method(*method),
                    );
                });
            },
        );
    }

    &group.finish();
}

fn bench_read_whole_all_methods<T>(mut group: BenchmarkGroup<T>, seeder: &dyn Seeder)
where
    T: Measurement,
{
    let size = 1024 * 1024;

    //
    for method in CompressionMethod::supported_methods().iter() {
        let mut buffer = Vec::with_capacity(size);
        let mut bytes = generate_whole_archive(
            &mut buffer,
            size,
            seeder,
            &FileOptions::default().compression_method(*method),
        );

        &group.bench_with_input(
            BenchmarkId::from_parameter(method),
            method,
            |bench, method| {
                bench.iter(|| {
                    let size = read_whole_archive(&buffer.as_slice());
                });
            },
        );
    }

    &group.finish();
}

fn bench_random_archive(bench: &mut Criterion) {
    let size = 1024 * 1024;

    bench_read_whole_all_methods(
        bench.benchmark_group("write_random_archive"),
        &RandomSeeder {},
    );
    bench_write_whole_all_methods(
        bench.benchmark_group("read_random_archive"),
        &RandomSeeder {},
    );
}

fn bench_lorem_archive(bench: &mut Criterion) {
    let size = 1024 * 1024;

    bench_read_whole_all_methods(
        bench.benchmark_group("write_lorem_archive"),
        &LoremSeeder {},
    );

    bench_write_whole_all_methods(bench.benchmark_group("read_lorem_archive"), &LoremSeeder {});
}

criterion_group!(benches, bench_random_archive, bench_lorem_archive);
criterion_main!(benches);
