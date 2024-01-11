#![feature(test)]

extern crate test;

use std::{env, fs, path::Path};

use test::{test_main, ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName, TestType};

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let mut tests = vec![];

    add_corpus_tests(
        &mut tests,
        "tests/calgary",
        CALGARY_COMPRESS_1_LENS,
        CALGARY_COMPRESS_999_LENS,
    );

    add_corpus_tests(
        &mut tests,
        "tests/silesia",
        SILESIA_COMPRESS_1_LENS,
        SILESIA_COMPRESS_999_LENS,
    );

    test_main(&args, tests, None);
}

fn add_corpus_tests(
    tests: &mut Vec<TestDescAndFn>,
    path: impl AsRef<Path>,
    compress_1_lens: &[(&str, usize)],
    compress_999_lens: &[(&str, usize)],
) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();

        let file_name = entry.file_name().to_str().unwrap().to_owned();

        let compress_1_len = compress_1_lens
            .iter()
            .find(|(name, _)| name == &file_name)
            .unwrap()
            .1;

        let path = entry.path();

        let test = TestDescAndFn {
            desc: TestDesc {
                name: TestName::DynTestName(format!("lzo1x-1 {file_name}")),
                ignore: false,
                ignore_message: None,
                source_file: "",
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
                should_panic: ShouldPanic::No,
                compile_fail: false,
                no_run: false,
                test_type: TestType::IntegrationTest,
            },
            testfn: TestFn::DynTestFn(Box::new(move || {
                test_roundtrip_1(path, compress_1_len);

                Ok(())
            })),
        };

        tests.push(test);

        let compress_999_len = compress_999_lens
            .iter()
            .find(|(name, _)| name == &file_name)
            .unwrap()
            .1;

        let test = TestDescAndFn {
            desc: TestDesc {
                name: TestName::DynTestName(format!("lzo1x-999 {file_name}")),
                ignore: false,
                ignore_message: None,
                source_file: "",
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
                should_panic: ShouldPanic::No,
                compile_fail: false,
                no_run: false,
                test_type: TestType::IntegrationTest,
            },
            testfn: TestFn::DynTestFn(Box::new(move || {
                test_roundtrip_999(entry.path(), compress_999_len);

                Ok(())
            })),
        };

        tests.push(test);
    }
}

fn test_roundtrip_1(path: impl AsRef<Path>, compressed_len: usize) {
    let data = fs::read(path.as_ref()).unwrap();

    let compressed = lzo1x::compress_1(&data);

    assert_eq!(compressed.len(), compressed_len);

    let mut decompressed = vec![0; data.len()];
    lzo1x::decompress(&compressed, &mut decompressed);

    assert!(decompressed == data);
}

fn test_roundtrip_999(path: impl AsRef<Path>, compressed_len: usize) {
    let data = fs::read(path.as_ref()).unwrap();

    let compressed = lzo1x::compress_999(&data);

    assert_eq!(compressed.len(), compressed_len);

    let mut decompressed = vec![0; data.len()];
    lzo1x::decompress(&compressed, &mut decompressed);

    assert!(decompressed == data);
}

const CALGARY_COMPRESS_1_LENS: &[(&str, usize)] = &[
    ("bib", 58272),
    ("book1", 495312),
    ("book2", 331307),
    ("geo", 100499),
    ("news", 215998),
    ("obj1", 12858),
    ("obj2", 117622),
    ("paper1", 28072),
    ("paper2", 46599),
    ("pic", 87126),
    ("progc", 19736),
    ("progl", 26586),
    ("progp", 17296),
    ("trans", 32241),
];

const SILESIA_COMPRESS_1_LENS: &[(&str, usize)] = &[
    ("dickens", 6207702),
    ("mozilla", 25451668),
    ("mr", 5335246),
    ("nci", 6282199),
    ("ooffice", 4159595),
    ("osdb", 5658863),
    ("reymont", 3170859),
    ("samba", 8017488),
    ("sao", 6469903),
    ("webster", 20036443),
    ("x-ray", 8497133),
    ("xml", 1292820),
];

const CALGARY_COMPRESS_999_LENS: &[(&str, usize)] = &[
    ("bib", 39332),
    ("book1", 356719),
    ("book2", 232407),
    ("geo", 74235),
    ("news", 160298),
    ("obj1", 11170),
    ("obj2", 88666),
    ("paper1", 20946),
    ("paper2", 33893),
    ("pic", 64504),
    ("progc", 15383),
    ("progl", 18743),
    ("progp", 12913),
    ("trans", 21608),
];

const SILESIA_COMPRESS_999_LENS: &[(&str, usize)] = &[
    ("dickens", 4402291),
    ("mozilla", 20705180),
    ("mr", 4044194),
    ("nci", 3789816),
    ("ooffice", 3312800),
    ("osdb", 4042468),
    ("reymont", 2125875),
    ("samba", 5880966),
    ("sao", 5771688),
    ("webster", 13693900),
    ("x-ray", 6767197),
    ("xml", 765173),
];
