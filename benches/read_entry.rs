use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main};
use criterion::{BenchmarkGroup, BenchmarkId, Criterion};

use std::io::{Cursor, Read, Write};
use std::ops::Add;
use std::str::from_utf8_mut;

use fake::faker::lorem::en::*;
use fake::{Dummy, Fake, Faker};
use rand::Rng;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

trait Seeder {
    // TODO: ...
    fn fill(&self, dest: &mut [u8]);

    fn reset(&mut self);
}

trait BucketSeeder {
    fn generate(&mut self);
    fn get_cache(&mut self) -> &'static mut Vec<u8>;
    fn get_index(&mut self) -> &'static mut usize;
    fn reset_index(&mut self);
}

impl Seeder for BucketSeeder {
    fn fill(&mut self, dest: &mut [u8]) {
        let mut cache = self.get_cache();

        loop {
            let remaining_cache = (cache.len() - self.get_index());
            if remaining_cache < dest.len() {
                break;
            }

            (self.generate());
        }

        dest.fill(&cache[self.get_index()..])
    }

    fn reset(&mut self) {
        self.reset_index();
    }
}

struct RandomSeeder {
    block_size: usize,
    index: usize,
}

impl Seeder for RandomSeeder {}
impl BucketSeeder for RandomSeeder {
    fn generate(&mut self) {
        let block = Vec<u8>::with_capacity(self.block_size);

        rand::thread_rng().fill(block);
        self.get_index().add(self.block_size)
    }
}

struct LoremSeeder {}

impl Seeder for LoremSeeder {}
impl BucketSeeder for LoremSeeder {
    fn fill(&self, dest: &mut [u8]) {
        // let mut buf = String::with_capacity(1024);
        // let count = 100;

        // for chunk in dest.chunks_mut(1024) {
        //     buf.clear();

        //     let paragraphs = Paragraphs(0..10).fake::<Vec<String>>();
        //     paragraphs.iter().map(|x| buf.push_str(x));

        //     // loop

        //     buf.truncate(1024);
        //     chunk.copy_from_slice(&buf.as_bytes());

        //     println!("{}", buf);
        // }
        // println!("length: {}", dest.len());
        // // dest.clear();
        // // dest.extend()
    }
}

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

fn bench_write_whole_all_methods<T>(mut group: BenchmarkGroup<T>, seeder: &dyn Seeder)
where
    T: Measurement,
{
    let size = 1024; // * 1024;

    //
    for method in CompressionMethod::supported_methods().iter() {
        let mut in_buffer = Vec::with_capacity(size);

        &group.bench_with_input(
            BenchmarkId::from_parameter(method),
            method,
            |bench, method| {
                in_buffer.clear();
                seeder.reset();

                bench.iter(|| {
                    let mut bytes = generate_whole_archive(
                        &mut in_buffer,
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
        let mut in_buffer = Vec::with_capacity(size);
        let mut bytes = generate_whole_archive(
            &mut in_buffer,
            size,
            seeder,
            &FileOptions::default().compression_method(*method),
        );

        &group.bench_with_input(
            BenchmarkId::from_parameter(method),
            method,
            |bench, method| {
                seeder.reset();

                bench.iter(|| {
                    let size = read_whole_archive(&in_buffer.as_slice());
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
        &mut RandomSeeder {
            block_size: 1024,
            index: 0
        },
    );

    bench_write_whole_all_methods(
        bench.benchmark_group("read_random_archive"),
        &mut RandomSeeder {
            block_size: 1024,
            index: 0
        },
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

criterion_group!(benches, bench_lorem_archive);
criterion_main!(benches);
