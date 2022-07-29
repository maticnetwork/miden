use super::{
    utils::range as create_range, Felt, FieldElement, Word, CHIPLETS_OFFSET,
    HASHER_AUX_TRACE_OFFSET,
};
use core::ops::Range;

pub mod bitwise;
pub mod hasher;
pub mod memory;

// CONSTANTS
// ================================================================================================

/// The number of columns in the chiplets which are used as selectors for the hasher chiplet.
pub const NUM_HASHER_SELECTORS: usize = 1;
/// The number of columns in the chiplets which are used as selectors for the bitwise chiplet.
pub const NUM_BITWISE_SELECTORS: usize = 2;
/// The number of columns in the chiplets which are used as selectors for the memory chiplet.
pub const NUM_MEMORY_SELECTORS: usize = 3;

/// The first column of the hash chiplet.
pub const HASHER_TRACE_OFFSET: usize = CHIPLETS_OFFSET + NUM_HASHER_SELECTORS;
/// The first column of the bitwise chiplet.
pub const BITWISE_TRACE_OFFSET: usize = CHIPLETS_OFFSET + NUM_BITWISE_SELECTORS;
/// The first column of the memory chiplet.
pub const MEMORY_TRACE_OFFSET: usize = CHIPLETS_OFFSET + NUM_MEMORY_SELECTORS;

// --- GLOBALLY-INDEXED CHIPLET COLUMN ACCESSORS --------------------------------------------------

/// The range within the main trace of the bitwise selector columns.
pub const BITWISE_SELECTOR_COL_RANGE: Range<usize> =
    create_range(BITWISE_TRACE_OFFSET, bitwise::NUM_SELECTORS);
/// The index within the main trace of the bitwise column holding the aggregated value of input `a`.
pub const BITWISE_A_COL_IDX: usize = BITWISE_TRACE_OFFSET + bitwise::A_COL_IDX;
/// The index within the main trace of the bitwise column holding the aggregated value of input `b`.
pub const BITWISE_B_COL_IDX: usize = BITWISE_TRACE_OFFSET + bitwise::B_COL_IDX;
/// The index range within the main trace for the bit decomposition of `a` for bitwise operations.
pub const BITWISE_A_COL_RANGE: Range<usize> = Range {
    start: BITWISE_TRACE_OFFSET + bitwise::A_COL_RANGE.start,
    end: BITWISE_TRACE_OFFSET + bitwise::A_COL_RANGE.end,
};
/// The index range within the main trace for the bit decomposition of `b` for bitwise operations.
pub const BITWISE_B_COL_RANGE: Range<usize> = Range {
    start: BITWISE_TRACE_OFFSET + bitwise::B_COL_RANGE.start,
    end: BITWISE_TRACE_OFFSET + bitwise::B_COL_RANGE.end,
};
/// The index within the main trace of the bitwise column containing the aggregated output value of
/// the previous row.
pub const BITWISE_PREV_OUTPUT_COL_IDX: usize = BITWISE_TRACE_OFFSET + bitwise::PREV_OUTPUT_COL_IDX;
/// The index within the main trace of the bitwise column containing the aggregated output value.
pub const BITWISE_OUTPUT_COL_IDX: usize = BITWISE_TRACE_OFFSET + bitwise::OUTPUT_COL_IDX;
