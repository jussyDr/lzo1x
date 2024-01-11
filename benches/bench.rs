#![feature(test)]

extern crate test;

use std::{fs::File, io::Read};

use test::Bencher;
use zip::ZipArchive;

#[bench]
fn compress_1(b: &mut Bencher) {
    let data = bench_data();

    b.iter(|| {
        lzo1x::compress(&data, 3).unwrap();
    })
}

#[bench]
fn compress_999(b: &mut Bencher) {
    let data = bench_data();

    b.iter(|| {
        lzo1x::compress(&data, 12).unwrap();
    })
}

#[bench]
fn decompress(b: &mut Bencher) {
    let data = bench_data();
    let compressed = lzo1x::compress(&data, 3).unwrap();

    let mut decompressed = vec![0; data.len()];

    b.iter(|| {
        lzo1x::decompress(&compressed, &mut decompressed);
    })
}

fn bench_data() -> Vec<u8> {
    let zip_file = File::open("corpora/calgary.zip").unwrap();
    let mut zip_archive = ZipArchive::new(zip_file).unwrap();
    let mut file = zip_archive.by_name("calgary/bib").unwrap();

    let mut data = vec![0; file.size() as usize];
    file.read_to_end(&mut data).unwrap();

    data
}
