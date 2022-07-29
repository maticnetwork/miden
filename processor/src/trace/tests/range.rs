use super::{build_trace_from_ops, Felt, FieldElement, Trace, NUM_RAND_ROWS};
use crate::{ONE, ZERO};
use rand_utils::rand_array;
use vm_core::{
    hasher::HASH_CYCLE_LEN,
    range::{P0_COL_IDX, P1_COL_IDX, Q_COL_IDX},
    Operation, AUX_TRACE_RAND_ELEMENTS,
};

#[test]
fn p0_trace() {
    // --- Range check 256_u32 (4 16-bit range checks: 0, 256 and 0, 0) ---------------------------
    let stack = [1, 255];
    let operations = vec![Operation::U32add];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let alpha = rand_elements[0];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let p0 = aux_columns.get_column(P0_COL_IDX);

    assert_eq!(trace.length(), p0.len());

    // 256 8-bit rows are needed to for each value 0-255. 64 8-bit rows are needed to check 256
    // increments of 255 in the 16-bit portion of the table, for a total of 256 + 63 = 319 rows.
    let len_8bit = 319;
    // 260 16-bit rows are needed for 0, 0, 255, 256, ... 255 increments of 255 ..., 65535. (0 is
    // range-checked in 2 rows for a total of 3 lookups. 256 is range-checked in one row. 65535 is
    // the max, and the rest are "bridge" values.)
    let len_16bit = 260;
    // The range checker is padded at the beginning, so the padding must be skipped.
    let start_8bit = trace.length() - len_8bit - len_16bit - NUM_RAND_ROWS;
    let start_16bit = trace.length() - len_16bit - NUM_RAND_ROWS;

    // The padded portion of the column should be all ones.
    let expected_padding = vec![ONE; start_8bit];
    assert_eq!(expected_padding, p0[..start_8bit]);

    // The first value in the 8-bit portion should be one.
    assert_eq!(ONE, p0[start_8bit]);

    // At the start of the 16-bit portion, the value of `p0` should include all the 8-bit lookups:
    // 1 lookup of zero; 1 lookup of one; 1 lookup of 254; 256 lookups of 255.
    // Therefore, the value should be: alpha * (alpha + 1) * (alpha + 254) + (alpha + 255)^256
    let mut acc_255 = alpha + Felt::new(255);
    for _ in 0..8 {
        acc_255 *= acc_255;
    }
    let expected_acc = alpha * (alpha + ONE) * (alpha + Felt::new(254)) * acc_255;
    assert_eq!(expected_acc, p0[start_16bit]);

    // The final value at the end of the 16-bit portion should be 1. This will be the last row
    // before the random row.
    assert_eq!(ONE, p0[p0.len() - 1 - NUM_RAND_ROWS]);
}

#[test]
#[allow(clippy::needless_range_loop)]
fn q_trace() {
    let stack = [1, 255];
    let operations = vec![
        Operation::U32add,
        Operation::MStoreW,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
    ];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let alpha = rand_elements[0];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let q = aux_columns.get_column(Q_COL_IDX);

    assert_eq!(trace.length(), q.len());

    // --- Check the stack processor's range check lookups. ---------------------------------------

    // Before any range checks are executed, the value in p1 should be one.
    assert_eq!(Felt::ONE, q[0]);

    // The first range check lookup from the stack will happen when the add operation is executed,
    // at cycle 1. (The trace begins by executing `span`). It must be divided out of `p1`.
    // The range-checked values are 0, 256, 0, 0.
    let expected = (alpha) * (Felt::new(256) + alpha) * alpha.square();
    assert_eq!(expected, q[1]);

    // --- Check the last value of the q column is one. ------------------------------------------

    for row in 2..(q.len() - NUM_RAND_ROWS) {
        assert_eq!(Felt::ONE, q[row]);
    }
}

/// This test checks that range check lookups from stack operations are balanced by the range checks
/// processed in the Range Checker.
///
/// The `U32add` operation results in 4 16-bit range checks of 256, 0, 0, 0.
#[test]
fn p1_trace_stack() {
    let stack = [1, 255];
    let operations = vec![Operation::U32add];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let alpha = rand_elements[0];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let p1 = aux_columns.get_column(P1_COL_IDX);

    assert_eq!(trace.length(), p1.len());

    // --- Check the stack processor's range check lookups. ---------------------------------------

    // Before any range checks are executed, the value in p1 should be one.
    assert_eq!(Felt::ONE, p1[0]);
    assert_eq!(Felt::ONE, p1[1]);

    // The first range check lookup from the stack will happen when the add operation is executed,
    // at cycle 1. (The trace begins by executing `span`). It must be divided out of `p1`.
    // The range-checked values are 0, 256, 0, 0.
    let lookup_product = (alpha) * (Felt::new(256) + alpha) * alpha.square();
    let mut expected = lookup_product.inv();
    assert_eq!(expected, p1[2]);

    // --- Check the range checker's lookups. -----------------------------------------------------

    // 260 16-bit rows are needed for 0, 0, 255, 256, ... 255 increments of 255 ..., 65535. (0 is
    // range-checked in 2 rows for a total of 3 lookups. 256 is range-checked in one row. 65535 is
    // the max, and the rest are "bridge" values.)
    let len_16bit = 260;
    // The start of the 16-bit section of the range checker table.
    let start_16bit = trace.length() - len_16bit - NUM_RAND_ROWS;

    // The values in p1 should not change again until the range checker's 16-bit table starts
    let expected_vec = vec![expected; start_16bit - 3];
    assert_eq!(expected_vec, p1[3..start_16bit]);

    // When the 16-bit portion of the table starts, the first value will be unchanged.
    assert_eq!(expected, p1[start_16bit]);
    // We include 2 lookups of 0, so the next value should be multiplied by alpha squared.
    expected *= alpha.square();
    assert_eq!(expected, p1[start_16bit + 1]);
    // Then we include our third lookup of 0, so the next value should be multiplied by alpha.
    expected *= alpha;
    assert_eq!(expected, p1[start_16bit + 2]);
    // Then we have a bridge row for 255 where the value does not change
    assert_eq!(expected, p1[start_16bit + 3]);
    // Then we include 1 lookup of 256, so it should be multiplied by alpha + 256.
    expected *= alpha + Felt::new(256);
    assert_eq!(expected, p1[start_16bit + 4]);

    // --- Check the last value of the p1 column is one. ------------------------------------------

    let last_row = p1.len() - NUM_RAND_ROWS - 1;
    assert_eq!(Felt::ONE, p1[last_row]);
}

/// This test checks that range check lookups from memory operations are balanced by the
/// range checks processed in the Range Checker.
///
/// The `StoreW` memory operation results in 2 16-bit range checks of 0, 0.
/// The `LoadW` memory operation results in 2 16-bit range checks of 0, 0.
#[test]
#[allow(clippy::needless_range_loop)]
fn p1_trace_mem() {
    let stack = [0, 1, 2, 3, 4, 0];
    let operations = vec![
        Operation::MStoreW,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::MLoadW,
    ];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let alpha = rand_elements[0];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let p1 = aux_columns.get_column(P1_COL_IDX);

    assert_eq!(trace.length(), p1.len());

    // The memory section of the chiplets trace starts after the span hash.
    let memory_start = HASH_CYCLE_LEN;
    // 260 16-bit rows are needed for 0, 0, 4, ... 256 increments of 255 ..., 65535. (0 is
    // range-checked in 2 rows for a total of 3 lookups. Four is range checked in one row for a
    // total of one lookup. 65535 is the max, and the rest are "bridge" values.)
    let len_16bit = 260;
    let start_16bit = trace.length() - len_16bit - NUM_RAND_ROWS;

    // The value should start at ONE and be unchanged until the memory processor section begins.
    let mut expected = ONE;
    for row in 0..=memory_start {
        assert_eq!(expected, p1[row]);
    }

    // --- Check the memory processor's range check lookups. --------------------------------------

    // There are two memory lookups. For each memory lookup, the context and address are unchanged,
    // so the delta values indicated the clock cycle change i' - i - 1.
    // StoreW is executed at cycle 1 (after the initial span), so i' - i - 1 = 0.
    let (d0_store, d1_store) = (ZERO, ZERO);
    // LoadW is executed at cycle 6, so i' - i - 1 = 6 - 1 - 1 = 4.
    let (d0_load, d1_load) = (Felt::new(4), ZERO);

    // Include the lookups from the `MStoreW` operation at the next row.
    expected *= ((d0_store + alpha) * (d1_store + alpha)).inv();
    assert_eq!(expected, p1[memory_start + 1]);
    // Include the lookup from the `MLoadW` operation at the next row.
    expected *= ((d0_load + alpha) * (d1_load + alpha)).inv();
    assert_eq!(expected, p1[memory_start + 2]);

    // The values in p1 should not change again until the range checker's 16-bit table starts
    for row in (memory_start + 3)..=start_16bit {
        assert_eq!(expected, p1[row]);
    }

    // --- Check the range checker's lookups. -----------------------------------------------------

    // We include 2 lookups of ZERO in the next row.
    expected *= alpha.square();
    assert_eq!(expected, p1[start_16bit + 1]);
    // We include 1 more lookup of ZERO in the next row.
    expected *= d0_store + alpha;
    assert_eq!(expected, p1[start_16bit + 2]);
    // We include 1 lookup of 4 in the next row.
    expected *= d0_load + alpha;
    assert_eq!(expected, p1[start_16bit + 3]);

    // --- The value should now be ONE for the rest of the trace. ---------------------------------
    assert_eq!(expected, ONE);
    for i in (start_16bit + 4)..(p1.len() - NUM_RAND_ROWS) {
        assert_eq!(ONE, p1[i]);
    }
}
