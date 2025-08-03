use lzo1x::{DecompressError, decompress};

#[test]
fn errors() {
    test_invalid_input(&[], &mut []);
    test_invalid_input(&[18], &mut []);
    test_output_length(&[18, 0], &mut []);
    test_invalid_input(&[18, 0], &mut [0]);
    test_invalid_input(&[0], &mut []);
    test_invalid_input(&[1], &mut []);
    test_output_length(&[1, 0, 0, 0, 0], &mut []);
    test_invalid_input(&[18, 0, 0], &mut [0]);
    test_invalid_input(&[16], &mut []);
    test_invalid_input(&[17, 0], &mut []);
    test_invalid_input(&[18, 0, 32], &mut [0]);
    test_invalid_input(&[18, 0, 33, 0], &mut [0]);
    test_invalid_input(&[18, 0, 64], &mut [0]);
    test_invalid_input(&[21, 0, 0, 0, 0, 0, 0], &mut [0, 0, 0, 0]);
    test_output_length(&[18, 0, 0, 0], &mut [0, 0]);
    test_invalid_input(&[18, 0, 1, 0], &mut [0, 0, 0]);
    test_output_length(&[18, 0, 1, 0, 0], &mut [0, 0, 0]);
    test_invalid_input(&[17, 0, 0, 0], &mut []);
    test_output_length(&[17, 0, 0], &mut [0]);
}

fn test_invalid_input(src: &[u8], dst: &mut [u8]) {
    let result = decompress(src, dst);

    assert_eq!(result, Err(DecompressError::InvalidInput));
}

fn test_output_length(src: &[u8], dst: &mut [u8]) {
    let result = decompress(src, dst);

    assert_eq!(result, Err(DecompressError::OutputLength));
}
