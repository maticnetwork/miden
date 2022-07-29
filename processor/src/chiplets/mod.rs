use super::{
    BTreeMap, ChipletsTrace, Felt, FieldElement, RangeChecker, StarkField, TraceFragment, Vec,
    Word, CHIPLETS_WIDTH, ONE, ZERO,
};
use crate::{trace::LookupTableRow, ExecutionError};
use core::ops::RangeInclusive;
use vm_core::{
    chiplets::bitwise::{BITWISE_AND_LABEL, BITWISE_OR_LABEL, BITWISE_XOR_LABEL},
    code_blocks::OpBatch,
};

mod bitwise;
use bitwise::{Bitwise, BitwiseLookup};

mod hasher;
pub use hasher::{AuxTraceBuilder as HasherAuxTraceBuilder, SiblingTableRow};
use hasher::{Hasher, HasherState};

mod memory;
use memory::{Memory, MemoryLookup};

mod bus;
pub use bus::{AuxTraceBuilder, ChipletsBus};

#[cfg(test)]
mod tests;

// CHIPLETS MODULE OF HASHER, BITWISE, AND MEMORY CHIPLETS
// ================================================================================================

/// This module manages the VM's hasher, bitwise, and memory chiplets and is responsible for
/// building a final execution trace from their stacked execution traces and chiplet selectors.
///
/// The module's trace can be thought of as 4 stacked chiplet segments in the following form:
/// * Hasher segment: contains the trace and selector for the hasher chiplet *
/// This segment fills the first rows of the trace up to the length of the hasher `trace_len`.
/// - column 0: selector column with values set to ZERO
/// - columns 1-17: execution trace of hash chiplet
///
/// * Bitwise segment: contains the trace and selectors for the bitwise chiplet *
/// This segment begins at the end of the hasher segment and fills the next rows of the trace for
/// the `trace_len` of the bitwise chiplet.
/// - column 0: selector column with values set to ONE
/// - column 1: selector column with values set to ZERO
/// - columns 2-15: execution trace of bitwise chiplet
/// - column 16-17: unused columns padded with ZERO
///
/// * Memory segment: contains the trace and selectors for the memory chiplet *
/// This segment begins at the end of the bitwise segment and fills the next rows of the trace for
/// the `trace_len` of the memory chiplet.
/// - column 0-1: selector columns with values set to ONE
/// - column 2: selector column with values set to ZERO
/// - columns 3-16: execution trace of memory chiplet
/// - column 17: unused column padded with ZERO
///
/// * Padding segment: unused *
/// This segment begins at the end of the memory segment and fills the rest of the execution trace
/// minus the number of random rows. When it finishes, the execution trace should have exactly
/// enough rows remaining for the specified number of random rows.
/// - columns 0-2: selector columns with values set to ONE
/// - columns 3-17: unused columns padded with ZERO
///
#[derive(Default)]
pub struct Chiplets {
    /// Current clock cycle of the VM.
    clk: usize,
    hasher: Hasher,
    bitwise: Bitwise,
    memory: Memory,
    bus: ChipletsBus,
}

impl Chiplets {
    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the length of the trace required to accommodate chiplet components and 1
    /// mandatory padding row required for ensuring sufficient trace length for auxiliary connector
    /// columns that rely on the memory chiplet.
    pub fn trace_len(&self) -> usize {
        self.hasher.trace_len() + self.bitwise.trace_len() + self.memory.trace_len() + 1
    }

    /// Returns the index of the first row of the [Memory] execution trace.
    pub fn memory_start(&self) -> usize {
        self.hasher.trace_len() + self.bitwise.trace_len()
    }

    // HASH CHIPLET ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Requests a single permutation of the hash function to the provided state from the Hash
    /// chiplet.
    ///
    /// The returned tuple contains the hasher state after the permutation and the row address of
    /// the execution trace at which the permutation started.
    pub fn permute(&mut self, state: HasherState) -> (Felt, HasherState) {
        self.hasher.permute(state)
    }

    /// Requests the hash of the provided words from the Hash chiplet and returns the result
    /// hash(h1, h2).
    ///
    /// The returned tuple also contains the row address of the execution trace at which hash
    /// computation started.
    pub fn merge(&mut self, h1: Word, h2: Word) -> (Felt, Word) {
        self.hasher.merge(h1, h2)
    }

    /// Requests computation a sequential hash of all operation batches in the list from the Hash
    /// chiplet and returns the result.
    ///
    /// The returned tuple also contains the row address of the execution trace at which hash
    /// computation started.
    pub fn hash_span_block(
        &mut self,
        op_batches: &[OpBatch],
        num_op_groups: usize,
    ) -> (Felt, Word) {
        self.hasher.hash_span_block(op_batches, num_op_groups)
    }

    /// Requests a Merkle root computation from the Hash chiplet for the specified path and the node
    /// with the specified value.
    ///
    /// The returned tuple contains the root of the Merkle path and the row address of the
    /// execution trace at which the computation started.
    ///
    /// # Panics
    /// Panics if:
    /// - The provided path does not contain any nodes.
    /// - The provided index is out of range for the specified path.
    pub fn build_merkle_root(&mut self, value: Word, path: &[Word], index: Felt) -> (Felt, Word) {
        self.hasher.build_merkle_root(value, path, index)
    }

    /// Requests a Merkle root update computation from the Hash chiplet.
    ///
    /// The returned tuple contains computed roots for the old value and the new value of the node
    /// with the specified path, as well as the row address of the execution trace at which the
    /// computation started.
    ///
    /// # Panics
    /// Panics if:
    /// - The provided path does not contain any nodes.
    /// - The provided index is out of range for the specified path.
    pub fn update_merkle_root(
        &mut self,
        old_value: Word,
        new_value: Word,
        path: &[Word],
        index: Felt,
    ) -> (Felt, Word, Word) {
        self.hasher
            .update_merkle_root(old_value, new_value, path, index)
    }

    // BITWISE CHIPLET ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Requests a bitwise AND of `a` and `b` from the Bitwise chiplet and returns the result.
    /// We assume that `a` and `b` are 32-bit values. If that's not the case, the result of the
    /// computation is undefined.
    pub fn u32and(&mut self, a: Felt, b: Felt) -> Result<Felt, ExecutionError> {
        let result = self.bitwise.u32and(a, b)?;

        let bitwise_lookup = BitwiseLookup::new(BITWISE_AND_LABEL, a, b, result);
        self.bus.request_bitwise_operation(bitwise_lookup, self.clk);

        Ok(result)
    }

    /// Requests a bitwise OR of `a` and `b` from the Bitwise chiplet and returns the result.
    /// We assume that `a` and `b` are 32-bit values. If that's not the case, the result of the
    /// computation is undefined.
    pub fn u32or(&mut self, a: Felt, b: Felt) -> Result<Felt, ExecutionError> {
        let result = self.bitwise.u32or(a, b)?;

        let bitwise_lookup = BitwiseLookup::new(BITWISE_OR_LABEL, a, b, result);
        self.bus.request_bitwise_operation(bitwise_lookup, self.clk);

        Ok(result)
    }

    /// Requests a bitwise XOR of `a` and `b` from the Bitwise chiplet and returns the result.
    /// We assume that `a` and `b` are 32-bit values. If that's not the case, the result of the
    /// computation is undefined.
    pub fn u32xor(&mut self, a: Felt, b: Felt) -> Result<Felt, ExecutionError> {
        let result = self.bitwise.u32xor(a, b)?;

        let bitwise_lookup = BitwiseLookup::new(BITWISE_XOR_LABEL, a, b, result);
        self.bus.request_bitwise_operation(bitwise_lookup, self.clk);

        Ok(result)
    }

    // MEMORY CHIPLET ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a word (4 elements) located in memory at the specified address.
    ///
    /// If the specified address hasn't been previously written to, four ZERO elements are
    /// returned. This effectively implies that memory is initialized to ZERO.
    pub fn read_mem(&mut self, addr: Felt) -> Word {
        // read the word from memory
        let value = self.memory.read(addr);

        // send the memory read request to the bus
        let memory_lookup = MemoryLookup::new(addr, self.clk as u64, value, value);
        self.bus.request_memory_operation(memory_lookup, self.clk);

        value
    }

    /// Writes the provided element to memory at the specified address leaving the remaining 3
    /// elements of the word previously stored at that address unchanged.
    pub fn write_mem_single(&mut self, addr: Felt, value: Felt) -> Word {
        let old_word = self.memory.get_old_value(addr);
        let word = [value, old_word[1], old_word[2], old_word[3]];

        self.memory.write(addr, word);

        // send the memory write request to the bus
        let memory_lookup = MemoryLookup::new(addr, self.clk as u64, old_word, word);
        self.bus.request_memory_operation(memory_lookup, self.clk);

        old_word
    }

    /// Writes the provided word (4 elements) to memory at the specified address.
    pub fn write_mem(&mut self, addr: Felt, word: Word) -> Word {
        let old_word = self.memory.get_old_value(addr);
        self.memory.write(addr, word);

        // send the memory write request to the bus
        let memory_lookup = MemoryLookup::new(addr, self.clk as u64, old_word, word);
        self.bus.request_memory_operation(memory_lookup, self.clk);

        old_word
    }

    /// Returns a word located at the specified address, or None if the address hasn't been
    /// accessed previously.
    pub fn get_mem_value(&self, addr: u64) -> Option<Word> {
        self.memory.get_value(addr)
    }

    /// Returns values within a range of addresses, or optionally all values at the beginning of.
    /// the specified cycle.
    /// TODO: Refactor to something like `pub fn get_mem_state(&self, clk: u64) -> Vec<(u64, Word)>`
    pub fn get_mem_values_at(&self, range: RangeInclusive<u64>, step: u64) -> Vec<(u64, Word)> {
        self.memory.get_values_at(range, step)
    }

    /// Returns current size of the memory (in words).
    #[cfg(test)]
    pub fn get_mem_size(&self) -> usize {
        self.memory.size()
    }

    // CONTEXT MANAGEMENT
    // --------------------------------------------------------------------------------------------

    /// Increments the clock cycle.
    pub fn advance_clock(&mut self) {
        self.memory.advance_clock();
        self.clk += 1;
    }

    // EXECUTION TRACE
    // --------------------------------------------------------------------------------------------

    /// Adds all range checks required by the memory chiplet to the provided [RangeChecker]
    /// instance, along with the cycle rows at which the processor performs the lookups.
    pub fn append_range_checks(&self, range_checker: &mut RangeChecker) {
        self.memory
            .append_range_checks(self.memory_start(), range_checker);
    }

    /// Returns an execution trace of the chiplets containing the stacked traces of the
    /// Hasher, Bitwise, and Memory chiplets.
    ///
    /// `num_rand_rows` indicates the number of rows at the end of the trace which will be
    /// overwritten with random values.
    pub fn into_trace(self, trace_len: usize, num_rand_rows: usize) -> ChipletsTrace {
        // make sure that only padding rows will be overwritten by random values
        assert!(
            self.trace_len() + num_rand_rows <= trace_len,
            "target trace length too small"
        );

        // Allocate columns for the trace of the chiplets.
        // note: it may be possible to optimize this by initializing with Felt::zeroed_vector,
        // depending on how the compiler reduces Felt(0) and whether initializing here + iterating
        // to update selector values is faster than using resize to initialize all values
        let mut trace = (0..CHIPLETS_WIDTH)
            .map(|_| Vec::<Felt>::with_capacity(trace_len))
            .collect::<Vec<_>>()
            .try_into()
            .expect("failed to convert vector to array");

        let (hasher_aux_builder, aux_builder) = self.fill_trace(&mut trace, trace_len);

        ChipletsTrace {
            trace,
            hasher_aux_builder,
            aux_builder,
        }
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    /// Fills the provided trace for the chiplets module with the stacked execution traces of the
    /// Hasher, Bitwise, and Memory chiplets, along with selector columns to identify each chiplet
    /// trace and padding to fill the rest of the trace.
    ///
    /// It returns the auxiilary trace builders for generating auxiliary trace columns that depend
    /// on data from [Chiplets].
    fn fill_trace(
        self,
        trace: &mut [Vec<Felt>; CHIPLETS_WIDTH],
        trace_len: usize,
    ) -> (HasherAuxTraceBuilder, AuxTraceBuilder) {
        // get the rows where chiplets begin.
        let bitwise_start = self.hasher.trace_len();
        let memory_start = self.memory_start();
        let Chiplets {
            clk: _,
            hasher,
            bitwise,
            memory,
            mut bus,
        } = self;

        // allocate fragments to be filled with the respective execution traces of each chiplet
        let mut hasher_fragment = TraceFragment::new(CHIPLETS_WIDTH);
        let mut bitwise_fragment = TraceFragment::new(CHIPLETS_WIDTH);
        let mut memory_fragment = TraceFragment::new(CHIPLETS_WIDTH);

        // set the selectors and padding as required by each column and segment
        // and add the hasher, bitwise, and memory segments to their respective fragments
        // so they can be filled with the chiplet traces
        for (column_num, column) in trace.iter_mut().enumerate() {
            match column_num {
                0 => {
                    // set the selector value for the hasher segment to ZERO
                    column.resize(hasher.trace_len(), Felt::ZERO);
                    // set the selector value for all other segments ONE
                    column.resize(trace_len, Felt::ONE);
                }
                1 => {
                    // initialize hasher segment and set bitwise segment selector value to ZERO
                    column.resize(hasher.trace_len() + bitwise.trace_len(), Felt::ZERO);
                    // set selector value for all other segments to ONE
                    column.resize(trace_len, Felt::ONE);
                    // add hasher segment to the hasher fragment to be filled from the hasher trace
                    hasher_fragment.push_column_slice(column, hasher.trace_len());
                }
                2 => {
                    // initialize hasher and bitwise segments and set memory segment selector to ZERO
                    column.resize(
                        hasher.trace_len() + bitwise.trace_len() + memory.trace_len(),
                        Felt::ZERO,
                    );
                    // set selector value for the final segment to ONE
                    column.resize(trace_len, Felt::ONE);
                    // add hasher segment to the hasher fragment to be filled from the hasher trace
                    let rest_of_column =
                        hasher_fragment.push_column_slice(column, hasher.trace_len());
                    // add bitwise segment to the bitwise fragment to be filled from the bitwise trace
                    bitwise_fragment.push_column_slice(rest_of_column, bitwise.trace_len());
                }
                16 => {
                    // initialize hasher & memory segments and bitwise, padding segments with ZERO
                    column.resize(trace_len, Felt::ZERO);
                    // add hasher segment to the hasher fragment to be filled from the hasher trace
                    let rest_of_column =
                        hasher_fragment.push_column_slice(column, hasher.trace_len());
                    // split the column to skip bitwise which have already been padded.
                    let (_, rest_of_column) = rest_of_column.split_at_mut(bitwise.trace_len());
                    // add memory segment to the memory fragment to be filled from the memory trace
                    memory_fragment.push_column_slice(rest_of_column, memory.trace_len());
                }
                17 => {
                    // initialize hasher segment and pad bitwise, memory, padding segments with ZERO
                    column.resize(trace_len, Felt::ZERO);
                    // add hasher segment to the hasher fragment to be filled from the hasher trace
                    hasher_fragment.push_column_slice(column, hasher.trace_len());
                }
                _ => {
                    // initialize hasher, bitwise, memory segments and pad the rest with ZERO
                    column.resize(trace_len, Felt::ZERO);
                    // add hasher segment to the hasher fragment to be filled from the hasher trace
                    let rest_of_column =
                        hasher_fragment.push_column_slice(column, hasher.trace_len());
                    // add bitwise segment to the bitwise fragment to be filled from the bitwise trace
                    let rest_of_column =
                        bitwise_fragment.push_column_slice(rest_of_column, bitwise.trace_len());
                    // add memory segment to the memory fragment to be filled from the memory trace
                    memory_fragment.push_column_slice(rest_of_column, memory.trace_len());
                }
            }
        }

        // fill the fragments with the execution trace from each chiplet
        // TODO: this can be parallelized to fill the traces in multiple threads
        let hasher_aux_builder = hasher.fill_trace(&mut hasher_fragment);
        bitwise.fill_trace(&mut bitwise_fragment, bitwise_start, &mut bus);
        memory.fill_trace(&mut memory_fragment, memory_start, &mut bus);

        (hasher_aux_builder, bus.into_aux_builder())
    }
}

// CHIPLETS LOOKUPS
// ================================================================================================

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum ChipletsLookup {
    Request(usize),
    Response(usize),
    RequestAndResponse((usize, usize)),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum ChipletsLookupRow {
    Hasher(HasherLookupRow),
    Bitwise(BitwiseLookup),
    Memory(MemoryLookup),
}

impl LookupTableRow for ChipletsLookupRow {
    fn to_value<E: FieldElement<BaseField = Felt>>(&self, alphas: &[E]) -> E {
        match self {
            ChipletsLookupRow::Hasher(row) => row.to_value(alphas),
            ChipletsLookupRow::Bitwise(row) => row.to_value(alphas),
            ChipletsLookupRow::Memory(row) => row.to_value(alphas),
        }
    }
}

// HASH PROCESSOR LOOKUPS
// ================================================================================================

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct HasherLookupRow {}

impl LookupTableRow for HasherLookupRow {
    /// Reduces this row to a single field element in the field specified by E. This requires
    /// at least 12 alpha values.
    fn to_value<E: FieldElement<BaseField = Felt>>(&self, _alphas: &[E]) -> E {
        unimplemented!()
    }
}
