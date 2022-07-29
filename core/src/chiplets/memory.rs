use super::{create_range, Felt, Range, MEMORY_TRACE_OFFSET};

// CONSTANTS
// ================================================================================================

/// The number of elements accessible in one read or write memory access.
pub const NUM_ELEMENTS: usize = 4;
/// Column to hold the context ID of the current memory context.
pub const CTX_COL_IDX: usize = MEMORY_TRACE_OFFSET;
/// Column to hold the memory address.
pub const ADDR_COL_IDX: usize = CTX_COL_IDX + 1;
/// Column for the clock cycle in which the memory operation occurred.
pub const CLK_COL_IDX: usize = ADDR_COL_IDX + 1;
/// Columns to hold the old values stored at a given memory context, address, and clock cycle prior
/// to the memory operation. When reading from a new address, these are initialized to zero. When
/// reading or updating previously accessed memory, these values are set to equal the "new" values
/// of the previous row in the trace.
pub const U_COL_RANGE: Range<usize> = create_range(CLK_COL_IDX + 1, NUM_ELEMENTS);
/// Columns to hold the new values stored at a given memory context, address, and clock cycle after
/// the memory operation.
pub const V_COL_RANGE: Range<usize> = create_range(U_COL_RANGE.end, NUM_ELEMENTS);
/// Column for the lower 16-bits of the delta between two consecutive context IDs, addresses, or
/// clock cycles.
pub const D0_COL_IDX: usize = V_COL_RANGE.end;
/// Column for the upper 16-bits of the delta between two consecutive context IDs, addresses, or
/// clock cycles.
pub const D1_COL_IDX: usize = D0_COL_IDX + 1;
/// Column for the inverse of the delta between two consecutive context IDs, addresses, or clock
/// cycles, used to enforce that changes are correctly constrained.
pub const D_INV_COL_IDX: usize = D1_COL_IDX + 1;

// --- OPERATION SELECTOR -----------------------------------------------------------------------

/// Unique label for memory operations. Computed as 1 more than the binary composition of the
/// chiplet selectors [1, 1, 1].
pub const MEMORY_LABEL: Felt = Felt::new(8);
