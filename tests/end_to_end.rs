use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashSet;
use std::io::prelude::*;
use std::io::{Cursor, Seek};
use std::iter::FromIterator;
use zip::read::ZipFile;
use zip::write::FileOptions;
use zip::{CompressionMethod, SUPPORTED_METHODS};

// This test asserts that after creating a zip file, then reading its contents back out,
// the extracted data will *always* be exactly the same as the original data.
#[test]
fn end_to_end() {
    for &method in SUPPORTED_METHODS {
        // Setup a buffer we can write and read an archive to.
        let buffer = &mut Cursor::new(Vec::new());

        // Write a test archive to the buffer.
        println!("Writing file with {} compression", method);
        write_test_archive(buffer, method).expect("Couldn't write test zip archive");

        // Load a fresh ZipArchive from the buffer data.
        let mut archive = zip::ZipArchive::new(buffer).expect("Couldn't load test archive");

        println!("Checking file contents");
        check_test_archive(&mut archive, Some(method));
    }
}

// This test asserts that after copying a `ZipFile` to a new `ZipWriter`, then reading its
// contents back out, the extracted data will *always* be exactly the same as the original data.
#[test]
fn copy() {
    for &method in SUPPORTED_METHODS {
        let buf_source = &mut Cursor::new(Vec::new());
        write_test_archive(buf_source, method).expect("Couldn't write to test file");

        let mut buf_target = &mut Cursor::new(Vec::new());

        {
            let mut archive_source = zip::ZipArchive::new(buf_source).unwrap();
            let mut zip = zip::ZipWriter::new(&mut buf_target);

            {
                let file = archive_source
                    .by_name(ENTRY_NAME)
                    .expect("Missing expected file");

                zip.raw_copy_file(file).expect("Couldn't copy file");
            }

            {
                let file = archive_source
                    .by_name(ENTRY_NAME)
                    .expect("Missing expected file");

                zip.raw_copy_file_rename(file, COPY_ENTRY_NAME)
                    .expect("Couldn't copy and rename file");
            }
        }

        let mut archive_target = zip::ZipArchive::new(buf_target).unwrap();
        check_test_archive(&mut archive_target, Some(method));
        check_archive_file(&mut archive_target, ENTRY_NAME, Some(method), LOREM_IPSUM);
        check_archive_file(
            &mut archive_target,
            COPY_ENTRY_NAME,
            Some(method),
            LOREM_IPSUM,
        );
    }
}

// This test asserts that after appending to a `ZipWriter`, then reading its contents back out,
// both the prior data and the appended data will be exactly the same as their originals.
#[test]
fn append() {
    for &method in SUPPORTED_METHODS {
        let mut file = &mut Cursor::new(Vec::new());
        write_test_archive(file, method).expect("Couldn't write to test file");

        {
            let mut zip = zip::ZipWriter::new_append(&mut file).unwrap();
            zip.start_file(
                COPY_ENTRY_NAME,
                FileOptions::default().compression_method(method),
            )
            .unwrap();
            zip.write_all(LOREM_IPSUM).unwrap();
            zip.finish().unwrap();
        }

        let mut zip = zip::ZipArchive::new(&mut file).unwrap();
        check_test_archive(&mut zip, Some(method));
        check_archive_file(&mut zip, ENTRY_NAME, Some(method), LOREM_IPSUM);
        check_archive_file(&mut zip, COPY_ENTRY_NAME, Some(method), LOREM_IPSUM);
    }
}

// Write a test zip archive to buffer.
fn write_test_archive(
    file: &mut Cursor<Vec<u8>>,
    method: CompressionMethod,
) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipWriter::new(file);

    zip.add_directory("test/", Default::default())?;

    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    zip.start_file("test/☃.txt", options)?;
    zip.write_all(b"Hello, World!\n")?;

    zip.start_file_with_extra_data(EXTRA_ENTRY_NAME, options)?;
    zip.write_u16::<LittleEndian>(0xbeef)?;
    zip.write_u16::<LittleEndian>(EXTRA_DATA.len() as u16)?;
    zip.write_all(EXTRA_DATA)?;
    zip.end_extra_data()?;
    zip.write_all(b"Hello, World! Again.\n")?;

    zip.start_file(ENTRY_NAME, options)?;
    zip.write_all(LOREM_IPSUM)?;

    zip.finish()?;

    Ok(())
}

// Load an archive from buffer and check for test data.
fn check_test_archive<R: Read + Seek>(archive: &mut zip::ZipArchive<R>, method: Option<CompressionMethod>) {
    check_archive_file(archive, ENTRY_NAME, method, LOREM_IPSUM);
    // check_archive_file(archive, EXTRA_ENTRY_NAME, method, LOREM_IPSUM);

    // Check archive contains expected file names.
    {
        let expected_file_names = [
            "test/",
            "test/☃.txt",
            EXTRA_ENTRY_NAME,
            ENTRY_NAME,
        ];
        let expected_file_names = HashSet::from_iter(expected_file_names.iter().copied());
        let file_names = archive.file_names().collect::<HashSet<_>>();
        assert_eq!(file_names, expected_file_names, "Archive file names differ");
    }

    // Check an archive file for extra data field contents.
    (|| -> zip::result::ZipResult<()> {
        let file_with_extra_data = archive.by_name(EXTRA_ENTRY_NAME)?;

        let mut extra_data = Vec::new();
        extra_data.write_u16::<LittleEndian>(0xbeef)?;
        extra_data.write_u16::<LittleEndian>(EXTRA_DATA.len() as u16)?;
        extra_data.write_all(EXTRA_DATA)?;

        assert_eq!(file_with_extra_data.extra_data(), extra_data.as_slice());

        Ok(())
    }())
    .expect("Could not check test file with extra data in archive");
}

// Check a file in the archive contains expected data and properties.
fn check_archive_file<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
    expected_method: Option<CompressionMethod>,
    expected_data: &[u8],
) {
    let mut file = archive.by_name(name).expect("Couldn not find file by name");

    if let Some(expected_method) = expected_method {
        // Check the file's compression method.
        let real_method = file.compression();

        assert_eq!(
            expected_method, real_method,
            "File does not have expected compression method"
        );
    }

    // Compare the file's contents.
    check_archive_file_contents(&mut file, expected_data);
}

// Check a file contains the given data.
fn check_archive_file_contents(file: &mut ZipFile, expected: &[u8]) {
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).unwrap();

    assert_eq!(file_contents.as_bytes(), expected);
}

const LOREM_IPSUM : &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tellus elit, tristique vitae mattis egestas, ultricies vitae risus. Quisque sit amet quam ut urna aliquet
molestie. Proin blandit ornare dui, a tempor nisl accumsan in. Praesent a consequat felis. Morbi metus diam, auctor in auctor vel, feugiat id odio. Curabitur ex ex,
dictum quis auctor quis, suscipit id lorem. Aliquam vestibulum dolor nec enim vehicula, porta tristique augue tincidunt. Vivamus ut gravida est. Sed pellentesque, dolor
vitae tristique consectetur, neque lectus pulvinar dui, sed feugiat purus diam id lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per
inceptos himenaeos. Maecenas feugiat velit in ex ultrices scelerisque id id neque.
";

const EXTRA_DATA: &[u8] = b"Extra Data";

const ENTRY_NAME: &str = "test/lorem_ipsum.txt";

const COPY_ENTRY_NAME: &str = "test/lorem_ipsum_renamed.txt";

const EXTRA_ENTRY_NAME: &str =" test_with_extra_data/🐢.txt";
