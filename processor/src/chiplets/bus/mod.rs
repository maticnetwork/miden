use super::{
    BTreeMap, BitwiseLookup, ChipletsLookup, ChipletsLookupRow, Felt, FieldElement, MemoryLookup,
    Vec,
};

mod aux_trace;
pub use aux_trace::AuxTraceBuilder;

// CHIPLETS BUS
// ================================================================================================

/// The Chiplets bus tracks data requested from or provided by chiplets in the Chiplets module. It
/// processes lookup requests from the stack & decoder and response data from the chiplets.
///
/// For correct execution, the lookup data used by the stack for each chiplet must be a permutation
/// of the lookups executed by that chiplet so that they cancel out. This is ensured by the `b_aux`
/// bus column. When the `b_aux` column is built, requests from the stack must be divided out and
/// lookup results provided by the chiplets must be multiplied in. To ensure that all lookups are
/// attributed to the correct chiplet and operation, a unique chiplet operation selector must be
/// included in the lookup row value when it is computed.
///
/// TODO: document the ChipletsBus components and their types.

#[derive(Default)]
pub struct ChipletsBus {
    lookup_hints: BTreeMap<usize, ChipletsLookup>,
    request_rows: Vec<ChipletsLookupRow>,
    response_rows: Vec<ChipletsLookupRow>,
}

impl ChipletsBus {
    // LOOKUP MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Requests a lookup at the specified cycle.
    fn request_operation(&mut self, cycle: usize) {
        // all requests are sent from the stack before responses are provided (during Chiplets trace
        // finalization). requests are guaranteed not to share cycles with other requests, since
        // only one operation will be executed at a time.
        let request_idx = self.request_rows.len();
        self.lookup_hints
            .insert(cycle, ChipletsLookup::Request(request_idx));
    }

    /// Provides lookup data at the specified cycle, which is the row of the AuxTable execution
    /// trace that contains this lookup row.
    fn provide_operation(&mut self, response_cycle: usize) {
        // results are guaranteed not to share cycles with other results, but they might share
        // a cycle with a request which has already been sent.
        let response_idx = self.response_rows.len();
        self.lookup_hints
            .entry(response_cycle)
            .and_modify(|lookup| {
                if let ChipletsLookup::Request(request_idx) = *lookup {
                    *lookup = ChipletsLookup::RequestAndResponse((request_idx, response_idx));
                }
            })
            .or_insert_with(|| ChipletsLookup::Response(response_idx));
    }

    // MEMORY LOOKUPS
    // --------------------------------------------------------------------------------------------

    /// Sends a request for the specified memory access. When `old_word` and `new_word` are the
    /// same in the MemoryLookup, this is a read request. When they are different, it's a write
    /// request. The memory value is requested at `cycle`. This request is expected to originate
    /// from operation executors.
    pub fn request_memory_operation(&mut self, lookup: MemoryLookup, cycle: usize) {
        self.request_operation(cycle);
        self.request_rows.push(ChipletsLookupRow::Memory(lookup));
    }

    /// Provides the data of the specified memory access.  When `old_word` and `new_word` are the
    /// same in the MemoryLookup, this is a read request. When they are different, it's a write  
    /// request. The memory access data is provided at cycle `response_cycle`, which is the row of
    /// the execution trace that contains this Memory row.
    pub fn provide_memory_operation(&mut self, lookup: MemoryLookup, response_cycle: usize) {
        self.provide_operation(response_cycle);
        self.response_rows.push(ChipletsLookupRow::Memory(lookup));
    }

    // BITWISE LOOKUPS
    // --------------------------------------------------------------------------------------------

    /// Requests the specified bitwise lookup at the specified `cycle`. This request is expected to
    /// originate from operation executors.
    pub fn request_bitwise_operation(&mut self, lookup: BitwiseLookup, cycle: usize) {
        self.request_operation(cycle);
        self.request_rows.push(ChipletsLookupRow::Bitwise(lookup));
    }

    /// Provides the data of a bitwise operation contained in the [Bitwise] table. The bitwise value
    /// is provided at cycle `response_cycle`, which is the row of the execution trace that contains
    /// this Bitwise row. It will always be the final row of a Bitwise operation cycle.
    pub fn provide_bitwise_operation(&mut self, lookup: BitwiseLookup, response_cycle: usize) {
        self.provide_operation(response_cycle);
        self.response_rows.push(ChipletsLookupRow::Bitwise(lookup));
    }

    // AUX TRACE BUILDER GENERATION
    // --------------------------------------------------------------------------------------------

    /// Converts this [ChipletsBus] into an auxiliary trace builder which can be used to construct
    /// the auxiliary trace column describing the [Chiplets] lookups at every cycle.
    pub fn into_aux_builder(self) -> AuxTraceBuilder {
        let lookup_hints = self.lookup_hints.into_iter().collect();

        AuxTraceBuilder {
            lookup_hints,
            request_rows: self.request_rows,
            response_rows: self.response_rows,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns an option with the lookup hint for the specified cycle.
    #[cfg(test)]
    pub(super) fn get_lookup_hint(&self, cycle: usize) -> Option<&ChipletsLookup> {
        self.lookup_hints.get(&cycle)
    }

    /// Returns the ith lookup response provided by the Chiplets module.
    #[cfg(test)]
    pub(super) fn get_response_row(&self, i: usize) -> ChipletsLookupRow {
        self.response_rows[i]
    }
}
