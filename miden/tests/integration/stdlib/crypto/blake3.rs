use super::{build_test, Felt, MIN_STACK_DEPTH};
use std::convert::TryInto;
use vm_core::utils::IntoBytes;

#[test]
fn blake3_2_to_1_hash() {
    let source = "
    use.std::crypto::hashes::blake3

    begin
        exec.blake3::hash
    end
    ";

    // prepare random input byte array
    let i_digest_0: [u8; 32] = rand_utils::rand_array::<Felt, 4>().into_bytes();
    let i_digest_1: [u8; 32] = rand_utils::rand_array::<Felt, 4>().into_bytes();

    let mut i_digest = [0u8; 64];
    i_digest[..32].copy_from_slice(&i_digest_0);
    i_digest[32..].copy_from_slice(&i_digest_1);

    // allocate space on stack so that bytes can be converted to blake3 words
    let mut i_words = [0u64; MIN_STACK_DEPTH];

    // convert each of four consecutive little endian bytes (of input) to blake3 words
    for (i, word) in i_words.iter_mut().enumerate().take(MIN_STACK_DEPTH) {
        let frm = i << 2;
        let to = (i + 1) << 2;
        *word = u32::from_le_bytes(i_digest[frm..to].try_into().unwrap()) as u64;
    }
    i_words.reverse();

    // use blake3 crate to compute 2-to-1 digest of byte array
    let digest = blake3::hash(&i_digest);

    // prepare digest in desired blake3 word form so that assertion writing becomes easier
    let digest_bytes = digest.as_bytes();
    let mut digest_words = [0u64; MIN_STACK_DEPTH >> 1];

    // convert each of four consecutive little endian bytes (of digest) to blake3 words
    for (i, word) in digest_words
        .iter_mut()
        .enumerate()
        .take(MIN_STACK_DEPTH >> 1)
    {
        let frm = i << 2;
        let to = (i + 1) << 2;
        *word = u32::from_le_bytes(digest_bytes[frm..to].try_into().unwrap()) as u64;
    }

    let test = build_test!(source, &i_words);
    test.expect_stack(&digest_words);
}
