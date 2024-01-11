#![feature(test)]

use std::{
    env,
    ffi::c_void,
    fs::{self, File},
    io::Read,
    mem::MaybeUninit,
    path::Path,
    sync::Arc,
};

use test::{test_main, ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName, TestType};
use zip::ZipArchive;

extern crate test;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let mut tests = vec![];

    add_corpus_tests(&mut tests, "corpora/calgary.zip");
    add_corpus_tests(&mut tests, "corpora/silesia.zip");

    add_fuzz_crash_test(
        &mut tests,
        "tests/fuzz/crash-8e855f271031b6ba31529bdd41d7c5571eec732c",
    );

    test_main(&args, tests, None);
}

fn add_corpus_tests(tests: &mut Vec<TestDescAndFn>, path: impl AsRef<Path>) {
    let file = File::open(path).unwrap();
    let mut zip_archive = ZipArchive::new(file).unwrap();

    for index in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(index).unwrap();

        if file.is_dir() {
            continue;
        }

        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf).unwrap();

        let data = Arc::new(buf);
        let data_ref = Arc::clone(&data);

        let test = create_test(
            format!("roundtrip 1 {}", file.name()),
            Box::new(move || {
                roundtrip_1(&data_ref);

                Ok(())
            }),
        );

        tests.push(test);

        let test = create_test(
            format!("roundtrip 999 {}", file.name()),
            Box::new(move || {
                roundtrip_999(&data);

                Ok(())
            }),
        );

        tests.push(test);
    }
}

fn add_fuzz_crash_test(tests: &mut Vec<TestDescAndFn>, path: &str) {
    let data = fs::read(path).unwrap();

    let test = create_test(
        format!("roundtrip 1 {path}"),
        Box::new(move || {
            roundtrip_1(&data);

            Ok(())
        }),
    );

    tests.push(test);
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

fn roundtrip_1(data: &[u8]) {
    let compressed = lzo1x::compress_1(data);

    assert!(compressed == lzo_sys_compress_1(data));

    let mut decompressed = vec![0; data.len()];
    lzo1x::decompress(&compressed, &mut decompressed);

    assert!(decompressed == data);
}

fn roundtrip_999(data: &[u8]) {
    let compressed = lzo1x::compress_999(data);

    assert!(compressed == lzo_sys_compress_999(data));

    let mut decompressed = vec![0; data.len()];
    lzo1x::decompress(&compressed, &mut decompressed);

    assert!(decompressed == data);
}

fn lzo_sys_compress_1(src: &[u8]) -> Vec<u8> {
    lzo_sys_compress(src, lzo_sys::lzo1x::lzo1x_1_compress)
}

fn lzo_sys_compress_999(src: &[u8]) -> Vec<u8> {
    lzo_sys_compress(src, lzo_sys::lzo1x::lzo1x_999_compress)
}

fn lzo_sys_compress(src: &[u8], compress_fn: lzo_sys::lzoconf::lzo_compress_t) -> Vec<u8> {
    let mut dst = vec![0; src.len() + (src.len() / 16) + 64 + 3];
    let mut dst_len = MaybeUninit::uninit();
    let mut wrkmem = vec![0; lzo_sys::lzo1x::LZO1X_1_MEM_COMPRESS as usize];

    unsafe {
        compress_fn(
            src.as_ptr(),
            src.len(),
            dst.as_mut_ptr(),
            dst_len.as_mut_ptr(),
            wrkmem.as_mut_ptr() as *mut c_void,
        )
    };

    let dst_len = unsafe { dst_len.assume_init() };

    dst.resize(dst_len, 0);

    dst
}
