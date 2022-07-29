use super::{utils::build_lookup_table_row_values, Felt, FieldElement, Matrix, Vec};
use crate::decoder::{AuxTraceHints, BlockTableUpdate, OpGroupTableUpdate};
use vm_core::{utils::uninit_vector, DECODER_TRACE_OFFSET};

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const ADDR_COL_IDX: usize = DECODER_TRACE_OFFSET + vm_core::decoder::ADDR_COL_IDX;

// DECODER AUXILIARY TRACE COLUMNS
// ================================================================================================

/// Builds and returns decoder auxiliary trace columns p1, p2, and p3 describing states of block
/// stack, block hash, and op group tables respectively.
pub fn build_aux_columns<E: FieldElement<BaseField = Felt>>(
    main_trace: &Matrix<Felt>,
    aux_trace_hints: &AuxTraceHints,
    rand_elements: &[E],
) -> Vec<Vec<E>> {
    let p1 = build_aux_col_p1(main_trace, aux_trace_hints, rand_elements);
    let p2 = build_aux_col_p2(main_trace, aux_trace_hints, rand_elements);
    let p3 = build_aux_col_p3(main_trace.num_rows(), aux_trace_hints, rand_elements);
    vec![p1, p2, p3]
}

// BLOCK STACK TABLE COLUMN
// ================================================================================================

/// Builds the execution trace of the decoder's `p1` column which describes the state of the block
/// stack table via multiset checks.
fn build_aux_col_p1<E: FieldElement<BaseField = Felt>>(
    main_trace: &Matrix<Felt>,
    aux_trace_hints: &AuxTraceHints,
    alphas: &[E],
) -> Vec<E> {
    // compute row values and their inverses for all rows that were added to the block stack table
    let table_rows = aux_trace_hints.block_stack_table_rows();
    let (row_values, inv_row_values) = build_lookup_table_row_values(table_rows, alphas);

    // allocate memory for the running product column and set the initial value to ONE
    let mut result = unsafe { uninit_vector(main_trace.num_rows()) };
    result[0] = E::ONE;

    // keep track of the index into the list of block stack table rows for started blocks; we can
    // use this index because the sequence in which blocks are started is exactly the same as the
    // sequence in which the rows are added to the block stack table.
    let mut started_block_idx = 0;

    // keep track of the last updated row in the running product column
    let mut result_idx = 0;

    // iterate through the list of updates and apply them one by one
    for (clk, update) in aux_trace_hints.block_exec_hints() {
        let clk = *clk;

        // if we skipped some cycles since the last update was processed, values in the last
        // updated row should by copied over until the current cycle.
        if result_idx < clk {
            let last_value = result[result_idx];
            result[(result_idx + 1)..=clk].fill(last_value);
        }

        // move the result pointer to the next row
        result_idx = clk + 1;

        // apply the relevant updates to the column
        match update {
            BlockTableUpdate::BlockStarted(_) => {
                // when a new block is started, multiply the running product by the value
                // representing the entry for the block in the block stack table.
                result[result_idx] = result[clk] * row_values[started_block_idx];
                started_block_idx += 1;
            }
            BlockTableUpdate::SpanExtended => {
                // when a RESPAN operation is executed, we need to remove the entry for
                // the last batch from the block stack table and also add an entry for the
                // new batch.
                let old_row_value_inv = inv_row_values[started_block_idx - 1];
                let new_row_value = row_values[started_block_idx];
                result[result_idx] = result[clk] * old_row_value_inv * new_row_value;
                started_block_idx += 1;
            }
            BlockTableUpdate::BlockEnded(_) => {
                // when a block is ended, we need to remove the entry for the block from the
                // block stack table; we can look up the index of the entry using the block's
                // ID which we get from the current row of the execution trace.
                let block_id = get_block_addr(main_trace, clk);
                let row_idx = aux_trace_hints
                    .get_block_stack_row_idx(block_id)
                    .expect("block stack row not found");
                result[result_idx] = result[clk] * inv_row_values[row_idx];
            }
            // REPEAT operation has no effect on the block stack table
            BlockTableUpdate::LoopRepeated => result[result_idx] = result[clk],
        }
    }

    // at this point, block stack table must be empty - so, the last value must be ONE;
    // we also fill in all the remaining values in the column with ONE's.
    let last_value = result[result_idx];
    assert_eq!(last_value, E::ONE);
    if result_idx < result.len() - 1 {
        result[(result_idx + 1)..].fill(E::ONE);
    }

    result
}

// BLOCK HASH TABLE COLUMN
// ================================================================================================

/// Builds the execution trace of the decoder's `p2` column which describes the state of the block
/// hash table via multiset checks.
fn build_aux_col_p2<E: FieldElement<BaseField = Felt>>(
    main_trace: &Matrix<Felt>,
    aux_trace_hints: &AuxTraceHints,
    alphas: &[E],
) -> Vec<E> {
    // compute row values and their inverses for all rows that were added to the block hash table
    let table_rows = aux_trace_hints.block_hash_table_rows();
    let (row_values, inv_row_values) = build_lookup_table_row_values(table_rows, alphas);

    // initialize memory for the running product column, and set the first value in the column to
    // the value of the first row (which represents an entry for the root block of the program)
    let mut result = unsafe { uninit_vector(main_trace.num_rows()) };
    result[0] = row_values[0];

    // keep track of the index into the list of block hash table rows for started blocks; we can
    // use this index because the sequence in which blocks are started is exactly the same as the
    // sequence in which the rows are added to the block hash table. we start at 1 because the
    // first row is already included in the running product above.
    let mut started_block_idx = 1;

    // keep track of the last updated row in the running product column
    let mut result_idx = 0;

    // iterate through the list of updates and apply them one by one
    for (clk, update) in aux_trace_hints.block_exec_hints() {
        let clk = *clk;

        // if we skipped some cycles since the last update was processed, values in the last
        // updated row should by copied over until the current cycle.
        if result_idx < clk {
            let last_value = result[result_idx];
            result[(result_idx + 1)..=clk].fill(last_value);
        }

        // move the result pointer to the next row
        result_idx = clk + 1;

        // apply relevant updates
        match update {
            BlockTableUpdate::BlockStarted(num_children) => {
                // if a new block was started, entries for the block's children are added to the
                // table; in case this was a JOIN block with two children, the first child should
                // have is_first_child set to true.
                match *num_children {
                    0 => result[result_idx] = result[clk],
                    1 => {
                        debug_assert!(!table_rows[started_block_idx].is_first_child());
                        result[result_idx] = result[clk] * row_values[started_block_idx];
                    }
                    2 => {
                        debug_assert!(table_rows[started_block_idx].is_first_child());
                        debug_assert!(!table_rows[started_block_idx + 1].is_first_child());
                        result[result_idx] = result[clk]
                            * row_values[started_block_idx]
                            * row_values[started_block_idx + 1];
                    }
                    _ => panic!("invalid number of children for a block"),
                }

                // move pointer into the table row list by the number of children
                started_block_idx += *num_children as usize;
            }
            BlockTableUpdate::LoopRepeated => {
                // When a REPEAT operation is executed, we need to add an entry for the loop's
                // body to the table. Entries for blocks in the block hash table can be identified
                // by their parent ID (which is the ID of the executing LOOP block). Parent ID is
                // always the address value in the next row of the execution trace after a REPEAT
                // operation is executed. Therefore, we can get the parent ID from the execution
                // trace at the next row: clk + 1 (which is the same as result_idx), and use it to
                // find this entry.
                let parent_id = get_block_addr(main_trace, result_idx);
                let row_idx = aux_trace_hints
                    .get_block_hash_row_idx(parent_id, false)
                    .expect("block hash row not found");
                result[result_idx] = result[clk] * row_values[row_idx];
            }
            BlockTableUpdate::BlockEnded(is_first_child) => {
                // when END operation is executed, we need to remove an entry for the block from
                // the block hash table. we can find the entry by its parent_id, which we can get
                // from the trace in the same way as described above. we also need to know whether
                // this block is the first or the second child of its parent, because for JOIN
                // block, the same parent ID would map to two children.
                let parent_id = get_block_addr(main_trace, result_idx);
                let row_idx = aux_trace_hints
                    .get_block_hash_row_idx(parent_id, *is_first_child)
                    .expect("block hash row not found");
                result[result_idx] = result[clk] * inv_row_values[row_idx];
            }
            // RESPAN operation has no effect on the block hash table
            BlockTableUpdate::SpanExtended => result[result_idx] = result[clk],
        }
    }

    // at this point, block hash table must be empty - so, the last value must be ONE;
    // we also fill in all the remaining values in the column with ONE's.
    let last_value = result[result_idx];
    assert_eq!(last_value, E::ONE);
    if result_idx < result.len() - 1 {
        result[(result_idx + 1)..].fill(E::ONE);
    }

    result
}

// OP GROUP TABLE COLUMN
// ================================================================================================

/// Builds the execution trace of the decoder's `p3` column which describes the state of the op
/// group table via multiset checks.
fn build_aux_col_p3<E: FieldElement<BaseField = Felt>>(
    trace_len: usize,
    aux_trace_hints: &AuxTraceHints,
    alphas: &[E],
) -> Vec<E> {
    // allocate memory for the column and set the starting value to ONE
    let mut result = unsafe { uninit_vector(trace_len) };
    result[0] = E::ONE;

    // compute row values and their inverses for all rows which were added to the op group table
    let (row_values, inv_row_values) =
        build_lookup_table_row_values(aux_trace_hints.op_group_table_rows(), alphas);

    // keep track of indexes into the list of op group table rows separately for inserted and
    // removed rows
    let mut inserted_group_idx = 0_usize;
    let mut removed_group_idx = 0_usize;

    // keep track of the last updated row in the running product column
    let mut result_idx = 0_usize;

    for (clk, update) in aux_trace_hints.op_group_table_hints() {
        let clk = *clk;

        // if we skipped some cycles since the last update was processed, values in the last
        // updated row should by copied over until the current cycle.
        if result_idx < clk {
            let last_value = result[result_idx];
            result[(result_idx + 1)..=clk].fill(last_value);
        }

        // apply the relevant updates to the column
        result_idx = clk + 1;
        match update {
            OpGroupTableUpdate::InsertRows(num_op_groups) => {
                // if the rows were added, multiply the current value in the column by the values
                // of all added rows
                let mut value = row_values[inserted_group_idx];
                for i in 1..(*num_op_groups as usize) {
                    value *= row_values[inserted_group_idx + i];
                }
                result[result_idx] = result[clk] * value;

                // advance the inserted group pointer by the number of inserted rows
                inserted_group_idx += *num_op_groups as usize;
            }
            OpGroupTableUpdate::RemoveRow => {
                // if a row was removed, divide the current value in the column by the value
                // of the row
                result[result_idx] = result[clk] * inv_row_values[removed_group_idx];

                // advance the removed group pointer by one
                removed_group_idx += 1;
            }
        }
    }

    // at this point, op group table must be empty - so, the last value must be ONE;
    // we also fill in all the remaining values in the column with ONE's.
    let last_value = result[result_idx];
    assert_eq!(last_value, E::ONE);
    if result_idx < result.len() - 1 {
        result[(result_idx + 1)..].fill(E::ONE);
    }

    result
}

// HELPER FUNCTIONS
// ================================================================================================

/// Returns the value in the block address column at the specified row.
fn get_block_addr(main_trace: &Matrix<Felt>, row_idx: usize) -> Felt {
    main_trace.get(ADDR_COL_IDX, row_idx)
}
