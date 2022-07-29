use super::{range, Felt, Operation, DECODER_AUX_TRACE_OFFSET, ONE, ZERO};
use core::ops::Range;

// CONSTANTS
// ================================================================================================

/// Index of the column holding code block IDs (which are row addresses from the hasher table).
pub const ADDR_COL_IDX: usize = 0;

/// Index at which operation bit columns start in the decoder trace.
pub const OP_BITS_OFFSET: usize = ADDR_COL_IDX + 1;

/// Number of columns needed to hold a binary representation of opcodes.
pub const NUM_OP_BITS: usize = Operation::OP_BITS;

/// Location of operation bits columns in the decoder trace.
pub const OP_BITS_RANGE: Range<usize> = range(OP_BITS_OFFSET, NUM_OP_BITS);

// TODO: probably rename "hasher state" to something like "shared columns".

/// Index at which hasher state columns start in the decoder trace.
pub const HASHER_STATE_OFFSET: usize = OP_BITS_OFFSET + NUM_OP_BITS;

/// Number of hasher columns in the decoder trace.
pub const NUM_HASHER_COLUMNS: usize = 8;

/// Number of helper registers available to user ops.
pub const NUM_USER_OP_HELPERS: usize = 6;

/// Index at which helper registers available to user ops start.
/// The first two helper registers are used by the decoder itself.
pub const USER_OP_HELPERS_OFFSET: usize = HASHER_STATE_OFFSET + 2;

/// Location of hasher columns in the decoder trace.
pub const HASHER_STATE_RANGE: Range<usize> = range(HASHER_STATE_OFFSET, NUM_HASHER_COLUMNS);

/// Index of the in_span column in the decoder trace.
pub const IN_SPAN_COL_IDX: usize = HASHER_STATE_OFFSET + NUM_HASHER_COLUMNS;

/// Index of the operation group count column in the decoder trace.
pub const GROUP_COUNT_COL_IDX: usize = IN_SPAN_COL_IDX + 1;

/// Index of the operation index column in the decoder trace.
pub const OP_INDEX_COL_IDX: usize = GROUP_COUNT_COL_IDX + 1;

/// Index at which operation batch flag columns start in the decoder trace.
pub const OP_BATCH_FLAGS_OFFSET: usize = OP_INDEX_COL_IDX + 1;

/// Number of operation batch flag columns.
pub const NUM_OP_BATCH_FLAGS: usize = 3;

/// Location of operation batch flag columns in the decoder trace.
pub const OP_BATCH_FLAGS_RANGE: Range<usize> = range(OP_BATCH_FLAGS_OFFSET, NUM_OP_BATCH_FLAGS);

/// Operation batch consists of 8 operation groups.
pub const OP_BATCH_8_GROUPS: [Felt; NUM_OP_BATCH_FLAGS] = [ONE, ZERO, ZERO];

/// Operation batch consists of 4 operation groups.
pub const OP_BATCH_4_GROUPS: [Felt; NUM_OP_BATCH_FLAGS] = [ZERO, ONE, ZERO];

/// Operation batch consists of 2 operation groups.
pub const OP_BATCH_2_GROUPS: [Felt; NUM_OP_BATCH_FLAGS] = [ZERO, ZERO, ONE];

/// Operation batch consists of 1 operation group.
pub const OP_BATCH_1_GROUPS: [Felt; NUM_OP_BATCH_FLAGS] = [ZERO, ONE, ONE];

// --- Column accessors in the auxiliary columns --------------------------------------------------

/// Running product column representing block stack table.
pub const P1_COL_IDX: usize = DECODER_AUX_TRACE_OFFSET;

/// Running product column representing block hash table
pub const P2_COL_IDX: usize = DECODER_AUX_TRACE_OFFSET + 1;

/// Running product column representing op group table.
pub const P3_COL_IDX: usize = DECODER_AUX_TRACE_OFFSET + 2;
