//! TODO: add docs

use super::{Felt, FieldElement, Word, HASHER_AUX_TRACE_OFFSET};
use core::ops::Range;
use crypto::{ElementHasher, Hasher as HashFn};

pub use crypto::hashers::Rp64_256 as Hasher;

// TYPES ALIASES
// ================================================================================================

/// Output type of Rescue Prime hash function.
///
/// The digest consists of 4 field elements or 32 bytes.
pub type Digest = <Hasher as HashFn>::Digest;

/// Type for Hasher trace selector. These selectors are used to define which transition function
/// is to be applied at a specific row of the hasher execution trace.
pub type Selectors = [Felt; NUM_SELECTORS];

// CONSTANTS
// ================================================================================================

/// Number of field element needed to represent the sponge state for the hash function.
///
/// This value is set to 12: 8 elements are reserved for rate and the remaining 4 elements are
/// reserved for capacity. This configuration enables computation of 2-to-1 hash in a single
/// permutation.
pub const STATE_WIDTH: usize = Hasher::STATE_WIDTH;

/// Number of field elements in the rate portion of the hasher's state.
pub const RATE_LEN: usize = 8;

/// Number of field elements in the capacity portion of the hasher's state.
pub const CAPACITY_LEN: usize = STATE_WIDTH - RATE_LEN;

// The length of the output portion of the hash state.
pub const DIGEST_LEN: usize = 4;

/// The output portion of the hash state, located in state elements 4, 5, 6, and 7.
pub const DIGEST_RANGE: Range<usize> = Hasher::DIGEST_RANGE;

/// Number of needed to complete a single permutation.
///
/// This value is set to 7 to target 128-bit security level with 40% security margin.
pub const NUM_ROUNDS: usize = Hasher::NUM_ROUNDS;

/// Number of selector columns in the trace.
pub const NUM_SELECTORS: usize = 3;

/// The number of rows in the execution trace required to compute a Rescue Prime permutation. This
/// is equal to 8.
pub const HASH_CYCLE_LEN: usize = NUM_ROUNDS.next_power_of_two();

/// Number of columns in Hasher execution trace. Additional two columns are for row address and
/// node index columns.
pub const TRACE_WIDTH: usize = NUM_SELECTORS + STATE_WIDTH + 2;

// --- Transition selectors -----------------------------------------------------------------------

/// Specifies a start of a new linear hash computation or absorption of new elements into an
/// executing linear hash computation. These selectors can also be used for a simple 2-to-1 hash
/// computation.
pub const LINEAR_HASH: Selectors = [Felt::ONE, Felt::ZERO, Felt::ZERO];
/// Unique label for the linear hash operation. Computed as 1 more than the binary composition of
/// the chiplet and operation selectors [0, 1, 0, 0].
pub const LINEAR_HASH_LABEL: Felt = Felt::new(3);

/// Specifies a start of Merkle path verification computation or absorption of a new path node
/// into the hasher state.
pub const MP_VERIFY: Selectors = [Felt::ONE, Felt::ZERO, Felt::ONE];
/// Unique label for the merkle path verification operation. Computed as 1 more than the binary
/// composition of the chiplet and operation selectors [0, 1, 0, 1].
pub const MP_VERIFY_LABEL: Felt = Felt::new(11);

/// Specifies a start of Merkle path verification or absorption of a new path node into the hasher
/// state for the "old" node value during Merkle root update computation.
pub const MR_UPDATE_OLD: Selectors = [Felt::ONE, Felt::ONE, Felt::ZERO];
/// Unique label for the merkle path update operation for an "old" node. Computed as 1 more than the
/// binary composition of the chiplet and operation selectors [0, 1, 1, 0].
pub const MR_UPDATE_OLD_LABEL: Felt = Felt::new(7);

/// Specifies a start of Merkle path verification or absorption of a new path node into the hasher
/// state for the "new" node value during Merkle root update computation.
pub const MR_UPDATE_NEW: Selectors = [Felt::ONE, Felt::ONE, Felt::ONE];
/// Unique label for the merkle path update operation for a "new" node. Computed as 1 more than the
/// binary composition of the chiplet and operation selectors [0, 1, 1, 1].
pub const MR_UPDATE_NEW_LABEL: Felt = Felt::new(15);

/// Specifies a completion of a computation such that only the hash result (values in h0, h1, h2
/// h3) is returned.
pub const RETURN_HASH: Selectors = [Felt::ZERO, Felt::ZERO, Felt::ZERO];
/// Unique label for specifying the return of a hash result. Computed as 1 more than the binary
/// composition of the chiplet and operation selectors [0, 0, 0, 0].
pub const RETURN_HASH_LABEL: Felt = Felt::new(1);

/// Specifies a completion of a computation such that the entire hasher state (values in h0 through
/// h11) is returned.
pub const RETURN_STATE: Selectors = [Felt::ZERO, Felt::ZERO, Felt::ONE];
/// Unique label for specifying the return of the entire hasher state. Computed as 1 more than the
/// binary composition of  the chiplet and operation selectors [0, 0, 0, 1]
pub const RETURN_STATE_LABEL: Felt = Felt::new(9);

// --- Column accessors in the auxiliary trace ----------------------------------------------------

/// Index of the auxiliary trace column tracking the state of the sibling table.
pub const P1_COL_IDX: usize = HASHER_AUX_TRACE_OFFSET;

// PASS-THROUGH FUNCTIONS
// ================================================================================================

/// Returns a hash of two digests. This method is intended for use in construction of Merkle trees.
#[inline(always)]
pub fn merge(values: &[Digest; 2]) -> Digest {
    Hasher::merge(values)
}

/// Returns a hash of the provided list of field elements.
#[inline(always)]
pub fn hash_elements(elements: &[Felt]) -> Digest {
    Hasher::hash_elements(elements)
}

/// Applies Rescue-XLIX round function to the provided state.
///
/// The function takes sponge state as an input and applies a single Rescue-XLIX round to it. The
/// round number must be specified via `round` parameter, which must be between 0 and 6 (both
/// inclusive).
#[inline(always)]
pub fn apply_round(state: &mut [Felt; STATE_WIDTH], round: usize) {
    Hasher::apply_round(state, round)
}

/// Applies Rescue-XLIX permutation (7 rounds) to the provided state.
#[inline(always)]
pub fn apply_permutation(state: &mut [Felt; STATE_WIDTH]) {
    Hasher::apply_permutation(state)
}

// HASHER STATE MUTATORS
// ================================================================================================

/// Initializes hasher state with the first 8 elements to be absorbed and the specified total
/// number of elements to be absorbed.
#[inline(always)]
pub fn init_state(init_values: &[Felt; RATE_LEN], num_elements: usize) -> [Felt; STATE_WIDTH] {
    [
        Felt::new(num_elements as u64),
        Felt::ZERO,
        Felt::ZERO,
        Felt::ZERO,
        init_values[0],
        init_values[1],
        init_values[2],
        init_values[3],
        init_values[4],
        init_values[5],
        init_values[6],
        init_values[7],
    ]
}

/// Initializes hasher state with the elements from the provided words. The number of elements
/// to be hashed is set to 8.
#[inline(always)]
pub fn init_state_from_words(w1: &Word, w2: &Word) -> [Felt; STATE_WIDTH] {
    [
        Felt::from(8_u8),
        Felt::ZERO,
        Felt::ZERO,
        Felt::ZERO,
        w1[0],
        w1[1],
        w1[2],
        w1[3],
        w2[0],
        w2[1],
        w2[2],
        w2[3],
    ]
}

/// Absorbs the specified values into the provided state by adding values to corresponding
/// elements in the rate portion of the state.
#[inline(always)]
pub fn absorb_into_state(state: &mut [Felt; STATE_WIDTH], values: &[Felt; RATE_LEN]) {
    state[4] += values[0];
    state[5] += values[1];
    state[6] += values[2];
    state[7] += values[3];
    state[8] += values[4];
    state[9] += values[5];
    state[10] += values[6];
    state[11] += values[7];
}

/// Returns elements representing the digest portion of the provided hasher's state.
pub fn get_digest(state: &[Felt; STATE_WIDTH]) -> [Felt; DIGEST_LEN] {
    state[DIGEST_RANGE]
        .try_into()
        .expect("failed to get digest from hasher state")
}
