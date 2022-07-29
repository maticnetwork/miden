use super::{build_test, Felt, MIN_STACK_DEPTH};
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use vm_core::utils::IntoBytes;

#[test]
fn sha256_2_to_1_hash() {
    let source = "
    use.std::crypto::hashes::sha256

    begin
        exec.sha256::hash
    end";

    // prepare random input byte array
    let i_digest_0: [u8; 32] = rand_utils::rand_array::<Felt, 4>().into_bytes();
    let i_digest_1: [u8; 32] = rand_utils::rand_array::<Felt, 4>().into_bytes();

    // two digests concatenated to form input to sha256 2-to-1 hash function
    let mut i_digest = [0u8; 64];
    i_digest[..32].copy_from_slice(&i_digest_0);
    i_digest[32..].copy_from_slice(&i_digest_1);

    // allocate space on stack so that bytes can be converted to sha256 words
    let mut i_words = [0u64; MIN_STACK_DEPTH];

    // convert each of four consecutive big endian bytes (of input) to sha256 words
    for (i, word) in i_words.iter_mut().enumerate().take(MIN_STACK_DEPTH) {
        let frm = i << 2;
        let to = (i + 1) << 2;
        *word = u32::from_be_bytes(i_digest[frm..to].try_into().unwrap()) as u64;
    }
    i_words.reverse();

    let mut hasher = Sha256::new();
    hasher.update(&i_digest);
    let digest = hasher.finalize();

    // prepare digest in desired sha256 word form so that assertion writing becomes easier
    let mut digest_words = [0u64; MIN_STACK_DEPTH >> 1];
    // convert each of four consecutive big endian bytes (of digest) to sha256 words
    for (i, word) in digest_words
        .iter_mut()
        .enumerate()
        .take(MIN_STACK_DEPTH >> 1)
    {
        let frm = i << 2;
        let to = (i + 1) << 2;
        *word = u32::from_be_bytes(digest[frm..to].try_into().unwrap()) as u64;
    }

    let test = build_test!(source, &i_words);
    test.expect_stack(&digest_words);
}
