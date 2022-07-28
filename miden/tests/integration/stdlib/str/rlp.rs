use super::build_test;
use rand_utils::rand_value;
use rlp;

struct FlagMemory {
    flag: bool,
    memory_address: Option<u32>,
}

// the decoding result should be stored on stack
#[test]
fn rlp_specific_value() {
    let source = "
    use.std::str::rlp

    begin
        exec.rlp::rlp_decode
    end
    ";
    let origin_str = "results are stored on stack";
    assert!(
        origin_str.len() <= 34,
        "the fixed-size convertion can only deal with string with lte 34 chars"
    );
    // flag - whether the decoding result should be stored into memory
    let flag = false;
    let flag_memory_on_stack = FlagMemory {
        flag,
        memory_address: None,
    };
    let rlp_encode_felt = comupute_rlp_encode_felt(origin_str, &flag_memory_on_stack);
    let expected_result = compute_expected_result(origin_str, &flag_memory_on_stack);

    build_test!(source, &[], &rlp_encode_felt, vec![]).expect_stack(&expected_result.0);
}

// the decoding result should be stored into memory
#[test]
fn rlp_specific_value_stored_into_memory() {
    let source = "
    use.std::str::rlp

    begin
        exec.rlp::rlp_decode
    end
    ";
    let origin_str = "results are stored in memory";
    assert!(
        origin_str.len() <= 34,
        "the fixed-size convertion can only deal with string with lte 34 chars"
    );
    // flag - whether the decoding result should be stored into memory
    let flag = true;

    // --- random u32 values ----------------------------------------------------------------------
    let address = rand_value::<u64>() as u32;

    let flag_memory_into_memory = FlagMemory {
        flag,
        memory_address: Some(address),
    };
    let rlp_encode_felt = comupute_rlp_encode_felt(origin_str, &flag_memory_into_memory);
    let expected_result = compute_expected_result(origin_str, &flag_memory_into_memory);

    build_test!(source, &[], &rlp_encode_felt, vec![]).expect_stack_and_memory(
        &expected_result.0,
        address as u64,
        &expected_result.1,
    );
}

// HELPER
fn comupute_rlp_encode_felt(origin_string: &str, flag_memory: &FlagMemory) -> Vec<u64> {
    // encoding the string into rlp code
    let mut rlp_encode_felt: Vec<u64> = Vec::new();
    let mut rlp_encode = rlp::encode(&origin_string).to_vec();

    // if the length of string is not some multiple of 7, pad with 0s
    for _i in 0..7 - (rlp_encode.len() % 7) {
        rlp_encode.push(0);
    }

    // reverse, make the padding '0's leading '0's, little-endian
    rlp_encode.reverse();
    let mut single_rlp_felt: u64 = 0;
    // combine every 7 rlp-element into a field element
    for (i, &item) in rlp_encode.iter().enumerate() {
        if i % 7 == 0 {
            single_rlp_felt = item as u64;
            continue;
        }
        single_rlp_felt = (single_rlp_felt as u64) * 256 + item as u64;
        if i % 7 == 6 {
            rlp_encode_felt.insert(0, single_rlp_felt);
        }
    }
    // fixed-size, the input should be 5 field element
    while rlp_encode_felt.len() < 5 {
        rlp_encode_felt.push(0);
    }
    // flag == false -- the result should be on stack, should insert '0'
    // flag == true -- the result should be stored into the memory address, should insert '1` at high-32-bit, and memory address at low-32-bit
    if flag_memory.flag == false {
        rlp_encode_felt.insert(0, 0);
    } else {
        rlp_encode_felt.insert(
            0,
            2_u64.pow(32) + flag_memory.memory_address.unwrap() as u64,
        );
    }
    return rlp_encode_felt;
}

// HELPER
fn compute_expected_result(origin_string: &str, flag_memory: &FlagMemory) -> (Vec<u64>, Vec<u64>) {
    let mut expected_result: Vec<u64> = Vec::new();
    let mut bytes = origin_string.as_bytes().to_vec();
    // pad '0's in the end, and reverse it making them become leading '0's
    for _i in 0..9 - (bytes.len() % 9) {
        bytes.push(0);
    }
    bytes.reverse();

    let mut ascii_felt: u64 = 0;
    for (i, &item) in bytes.iter().enumerate() {
        if i % 9 == 0 {
            ascii_felt = item as u64;
            continue;
        }
        ascii_felt = (ascii_felt as u64) * 128 + item as u64;
        if i % 9 == 8 {
            expected_result.insert(0, ascii_felt);
        }
    }
    let mut decoding_result = expected_result.clone();
    decoding_result.reverse();

    // flag == false -- the result should be on stack
    // flag == true -- the result should be stored into the memory address, should insert '1` at high-32-bit, and memory address at low-32-bit
    if flag_memory.flag == false {
        expected_result.insert(0, 0);
    } else {
        // the decoding result should be stored into memery
        expected_result.clear();
        expected_result.push(2_u64.pow(32) + flag_memory.memory_address.unwrap() as u64);
    }
    expected_result.insert(0, origin_string.len() as u64);

    return (expected_result, decoding_result);
}
