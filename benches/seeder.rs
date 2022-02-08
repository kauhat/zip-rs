use fake::faker::lorem::en::*;
use fake::{Dummy, Fake, Faker};
use getrandom::getrandom;
use std::io::{Seek, Write};
use zip::result::ZipResult;
use zip::write::FileOptions;

/// Generate fake files and metadata for testing zip archives.
///
/// TODO
#[derive(Debug, PartialEq)]
pub struct ArchiveFaker {}

impl ArchiveFaker {
    pub fn file_name() -> String {
        return unimplemented!();
    }

    // This method will help users to discover the builder
    pub fn builder() -> ArchiveBuilder {
        ArchiveBuilder::default()
    }
}

/// Friendly builder used to generate entire fake archives.
///
/// TODO
#[derive(Default)]
pub struct ArchiveBuilder {
    // Probably lots of optional fields.
    name_generator: Fn() -> String,
    file_options_generator: Fn() -> FileOptions,
    file_data_generator: Fn() -> &[u8],
}

impl ArchiveBuilder {
    pub fn new() -> ArchiveBuilder {
        ArchiveBuilder {
            name_generator: ArchiveFaker::file_name,
            file_options_generator: || -> FileOptions { FileOptions::default() },
            file_data_generator: || -> &[u8] { vec![] },
        }
    }

    pub fn file_names_from<F: Fn() -> String>(mut self, bar: F) -> ArchiveBuilder {
        self.name_generator = bar;
        self
    }

    pub fn file_options_from<F: Fn() -> FileOptions>(mut self, bar: F) -> ArchiveBuilder {
        self.file_data_generator = bar;
        self
    }

    pub fn file_data_from<F: Fn() -> &[u8]>(mut self, bar: F) -> ArchiveBuilder {
        self.file_data_generator = bar;
        self
    }

    pub fn generate<T: Write + Seek>(self, file: T) {
        // TODO
        return unimplemented!();
    }
}

/// Provide test data generated before benchmarks.
///
/// TODO
///
/// - Seeder has "bucket".
/// - Bucket is pre-generated with data of a certain size
///     (e.g. the size of all files in a test archive)
/// - Bucket is re-used after bench iteration.
/// - When calling fill(), the seeder will slice from the bucket's data and increment the index.
/// - When re-using a bucket it's index is reset.
/// - If a bucket runs out of pre-generated data:
///     - A warning is displayed, but new data is still generated.
///     - Subsequent iterations shouldn't run out of data.
trait Seeder {
    // TODO: ...
    fn fill(&self, dest: &mut [u8]);

    fn reset(&mut self);
}

trait BucketSeeder {
    fn generate(&mut self, size: usize) -> [u8];
    // fn get_cache(&mut self) -> &'static mut Vec<u8>;
    // fn get_index(&mut self) -> &'static mut usize;
    // fn reset_index(&mut self);
}

// impl Seeder for BucketSeeder {
//     fn fill(&mut self, dest: &mut [u8]) {
//         let mut cache = self.get_cache();

//         loop {
//             let remaining_cache = (cache.len() - self.get_index());
//             if remaining_cache < dest.len() {
//                 break;
//             }

//             (self.generate());
//         }

//         dest.fill(&cache[self.get_index()..])
//     }

//     fn reset(&mut self) {
//         self.reset_index();
//     }
// }

pub struct RandomSeeder {
    block_size: usize,
    index: usize,
}

impl Seeder for RandomSeeder {}

impl BucketSeeder for RandomSeeder {
    fn generate(&mut self, size: usize) {
        let block = Vec::<u8>::with_capacity(self.block_size);

        let mut bytes = vec![0u8; size];
        -getrandom(&mut bytes).unwrap();

        self.get_index().add(self.block_size)
    }
}

pub struct LoremSeeder {}

impl Seeder for LoremSeeder {}

impl BucketSeeder for LoremSeeder {
    fn generate(&mut self, size: usize) -> [u8] {
        return unimplemented!();
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
