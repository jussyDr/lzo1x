#![feature(test)]

extern crate test;

use std::{env, fs, path::Path};

use test::{test_main, ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName, TestType};

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let mut tests = vec![];

    add_corpus_tests(&mut tests, "tests/calgary");
    add_corpus_tests(&mut tests, "tests/silesia");

    test_main(&args, tests, None);
}

fn add_corpus_tests(tests: &mut Vec<TestDescAndFn>, path: impl AsRef<Path>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();

        let file_name = entry.file_name().to_str().unwrap().to_owned();

        let test_1 = create_test(
            format!("lzo1x-1 {file_name}"),
            Box::new(move || {
                test_roundtrip_1(entry.path());

                Ok(())
            }),
        );

        tests.push(test_1);
    }
}

fn create_test(
    name: String,
    test_fn: Box<dyn FnOnce() -> Result<(), String> + Send>,
) -> TestDescAndFn {
    TestDescAndFn {
        desc: TestDesc {
            name: TestName::DynTestName(name),
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
        testfn: TestFn::DynTestFn(test_fn),
    }
}

fn test_roundtrip_1(path: impl AsRef<Path>) {
    let data = fs::read(path).unwrap();

    let compressed = lzo1x::compress_1(&data);

    let mut decompressed = vec![0; data.len()];
    lzo1x::decompress(&compressed, &mut decompressed).unwrap();

    assert!(decompressed == data);
}
