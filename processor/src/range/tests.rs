use crate::{utils::get_trace_len, RangeCheckTrace};

use super::{BTreeMap, Felt, FieldElement, RangeChecker};
use rand_utils::rand_array;
use vm_core::{utils::ToElements, StarkField};

#[test]
fn range_checks() {
    let mut checker = RangeChecker::new();

    let values = [0, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 100, 355, 620].to_elements();

    for &value in values.iter() {
        checker.add_value(value.as_int() as u16)
    }

    let RangeCheckTrace {
        trace,
        aux_builder: _,
    } = checker.into_trace(1024, 0);
    validate_trace(&trace, &values);

    // skip the 8-bit portion of the trace
    let mut i = 0;
    while trace[0][i] == Felt::ZERO {
        i += 1
    }

    // make sure the values are arranged as expected
    validate_row(&trace, i, 0, 1);
    validate_row(&trace, i + 1, 1, 1);
    validate_row(&trace, i + 2, 2, 4);
    validate_row(&trace, i + 3, 3, 2);
    validate_row(&trace, i + 4, 3, 1);
    validate_row(&trace, i + 5, 4, 2);
    validate_row(&trace, i + 6, 100, 1);
    validate_row(&trace, i + 7, 355, 1);
    validate_row(&trace, i + 8, 610, 0);
    validate_row(&trace, i + 9, 620, 1);
}

#[test]
fn range_checks_rand() {
    let mut checker = RangeChecker::new();
    let values = rand_array::<u64, 300>();
    let values = values
        .into_iter()
        .map(|v| Felt::new(v as u16 as u64))
        .collect::<Vec<_>>();

    for &value in values.iter() {
        checker.add_value(value.as_int() as u16);
    }

    let trace_len = checker.trace_len().next_power_of_two();
    let RangeCheckTrace {
        trace,
        aux_builder: _,
    } = checker.into_trace(trace_len, 0);
    validate_trace(&trace, &values);
}

// HELPER FUNCTIONS
// ================================================================================================

fn validate_row(trace: &[Vec<Felt>], row_idx: usize, value: u64, num_lookups: u64) {
    let (s0, s1) = match num_lookups {
        0 => (Felt::ZERO, Felt::ZERO),
        1 => (Felt::ONE, Felt::ZERO),
        2 => (Felt::ZERO, Felt::ONE),
        4 => (Felt::ONE, Felt::ONE),
        _ => panic!("invalid lookup value"),
    };

    assert_eq!(s0, trace[1][row_idx]);
    assert_eq!(s1, trace[2][row_idx]);
    assert_eq!(Felt::new(value), trace[3][row_idx]);
}

fn validate_trace(trace: &[Vec<Felt>], lookups: &[Felt]) {
    assert_eq!(4, trace.len());

    // trace length must be a power of two
    let trace_len = get_trace_len(trace);
    assert!(trace_len.is_power_of_two());

    // --- validate the 8-bit segment of the trace ----------------------------

    // should start with a ZERO
    assert_eq!(Felt::ZERO, trace[0][0]);
    assert_eq!(Felt::ZERO, trace[3][0]);

    let mut lookups_8bit = BTreeMap::new();

    // values in the 8-bit segment should be 8 bits; also, count the number of lookups
    // for each 8-bit value.
    let mut i = 0;
    let mut prev_value = 0u8;
    while trace[0][i] == Felt::ZERO {
        // make sure the value is an 8-bit value
        let value = trace[3][i].as_int();
        assert!(value <= 255, "not an 8-bit value");
        let value = value as u8;

        // make sure the delta between this and the previous value is either 0 or 1
        let delta = value - prev_value;
        assert!(delta <= 1);

        // keep track of lookup count for each value
        let count = get_lookup_count(trace, i);
        lookups_8bit
            .entry(value)
            .and_modify(|value| *value += count)
            .or_insert(count);

        i += 1;
        prev_value = value;
    }

    // validate the last row (must be 255)
    let last_value = trace[3][i - 1].as_int();
    assert_eq!(255, last_value);

    // all possible 8-bit values must be in the map
    assert_eq!(256, lookups_8bit.len());

    // --- validate the 16-bit segment of the trace ---------------------------

    let mut lookups_16bit = BTreeMap::new();

    // process the first row
    assert_eq!(Felt::ZERO, trace[3][i]);
    let count = get_lookup_count(trace, i);
    lookups_16bit.insert(0u16, count);
    i += 1;

    // process all other rows
    let mut prev_value = 0u16;
    while i < trace_len {
        // segment identifier must be set to ONE
        assert_eq!(Felt::ONE, trace[0][i]);

        // make sure the value is a 16-bit value
        let value = trace[3][i].as_int();
        assert!(value <= 65535, "not an 8-bit value");
        let value = value as u16;

        // make sure the delta between this and the previous value is less than 255,
        // and remove the delta from the 8-bit table
        let delta = value - prev_value;
        assert!(delta <= 255);
        let delta = delta as u8;
        lookups_8bit.entry(delta).and_modify(|count| {
            assert!(*count > 0);
            *count -= 1;
        });

        // keep track of lookup count for each value
        let count = get_lookup_count(trace, i);
        lookups_16bit
            .entry(value)
            .and_modify(|value| *value += count)
            .or_insert(count);

        i += 1;
        prev_value = value;
    }

    // validate the last row (must be 65535)
    let last_value = trace[3][i - 1].as_int();
    assert_eq!(65535, last_value);

    // at the end, 8-bit table should be empty
    for &value in lookups_8bit.values() {
        assert_eq!(0, value);
    }

    // remove all the looked up values from the 16-bit lookup table
    for value in lookups {
        let value = value.as_int();
        assert!(value <= 65535, "not a 16-bit value");
        let value = value as u16;

        assert!(lookups_16bit.contains_key(&value));
        lookups_16bit.entry(value).and_modify(|count| {
            assert!(*count > 0);
            *count -= 1;
        });
    }

    // make sure 16-bit table is empty
    for &value in lookups_16bit.values() {
        assert_eq!(0, value);
    }
}

fn get_lookup_count(trace: &[Vec<Felt>], step: usize) -> usize {
    if trace[1][step] == Felt::ZERO && trace[2][step] == Felt::ZERO {
        0
    } else if trace[1][step] == Felt::ONE && trace[2][step] == Felt::ZERO {
        1
    } else if trace[1][step] == Felt::ZERO && trace[2][step] == Felt::ONE {
        2
    } else if trace[1][step] == Felt::ONE && trace[2][step] == Felt::ONE {
        4
    } else {
        panic!("not a valid count");
    }
}
