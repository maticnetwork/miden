use super::{Felt, Join, Loop, OpBatch, Operation, Span, Split};

// DECODER
// ================================================================================================
pub struct Decoder {}

impl Decoder {
    pub fn new() -> Self {
        Self {}
    }

    // JOIN BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_join(&mut self, _block: &Join) {}

    pub fn end_join(&mut self, _block: &Join) {}

    // SPLIT BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_split(&mut self, _block: &Split, _condition: Felt) {}

    pub fn end_split(&mut self, _block: &Split) {}

    // LOOP BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_loop(&mut self, _block: &Loop, _condition: Felt) {}

    pub fn repeat(&mut self, _block: &Loop) {}

    pub fn end_loop(&mut self, _block: &Loop) {}

    // SPAN BLOCK
    // --------------------------------------------------------------------------------------------
    pub fn start_span(&mut self, _block: &Span) {}

    pub fn respan(&mut self, _op_batch: &OpBatch) {}

    pub fn execute_user_op(&mut self, _op: &Operation) {}

    pub fn end_span(&mut self, _block: &Span) {}
}
