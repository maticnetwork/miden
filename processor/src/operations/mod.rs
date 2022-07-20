use super::{ExecutionError, Felt, FieldElement, Operation, Process, StarkField};

mod crypto_ops;
mod field_ops;
mod io_ops;
mod stack_ops;
mod sys_ops;
mod u32_ops;
mod utils;

// OPERATION DISPATCHER
// ================================================================================================

impl Process {
    /// Executes the specified operation.
    pub(super) fn execute_op(&mut self, op: Operation) -> Result<(), ExecutionError> {
        // make sure there is enough memory allocated to hold the execution trace
        self.ensure_trace_capacity();

        // execute the operation
        match op {
            // ----- system operations ------------------------------------------------------------
            Operation::Noop => self.stack.copy_state(0),
            Operation::Assert => self.op_assert()?,

            // ----- flow control operations ------------------------------------------------------
            // control flow operations are never executed directly
            Operation::Join => unreachable!("control flow operation"),
            Operation::Split => unreachable!("control flow operation"),
            Operation::Loop => unreachable!("control flow operation"),
            Operation::Repeat => unreachable!("control flow operation"),
            Operation::Span => unreachable!("control flow operation"),
            Operation::Respan => unreachable!("control flow operation"),
            Operation::End => unreachable!("control flow operation"),
            Operation::Halt => unreachable!("control flow operation"),

            // ----- field operations -------------------------------------------------------------
            Operation::Add => self.op_add()?,
            Operation::Neg => self.op_neg()?,
            Operation::Mul => self.op_mul()?,
            Operation::Inv => self.op_inv()?,
            Operation::Incr => self.op_incr()?,

            Operation::And => self.op_and()?,
            Operation::Or => self.op_or()?,
            Operation::Not => self.op_not()?,

            Operation::Eq => self.op_eq()?,
            Operation::Eqz => self.op_eqz()?,
            Operation::Eqw => self.op_eqw()?,
            Operation::BinAcc => self.op_binacc()?,

            // ----- u32 operations ---------------------------------------------------------------
            Operation::U32split => self.op_u32split()?,
            Operation::U32add => self.op_u32add()?,
            Operation::U32add3 => self.op_u32add3()?,
            Operation::U32sub => self.op_u32sub()?,
            Operation::U32mul => self.op_u32mul()?,
            Operation::U32madd => self.op_u32madd()?,
            Operation::U32div => self.op_u32div()?,

            Operation::U32and => self.op_u32and()?,
            Operation::U32or => self.op_u32or()?,
            Operation::U32xor => self.op_u32xor()?,
            Operation::U32assert2 => self.op_u32assert2()?,

            // ----- stack manipulation -----------------------------------------------------------
            Operation::Pad => self.op_pad()?,
            Operation::Drop => self.op_drop()?,

            Operation::Dup0 => self.op_dup(0)?,
            Operation::Dup1 => self.op_dup(1)?,
            Operation::Dup2 => self.op_dup(2)?,
            Operation::Dup3 => self.op_dup(3)?,
            Operation::Dup4 => self.op_dup(4)?,
            Operation::Dup5 => self.op_dup(5)?,
            Operation::Dup6 => self.op_dup(6)?,
            Operation::Dup7 => self.op_dup(7)?,
            Operation::Dup9 => self.op_dup(9)?,
            Operation::Dup11 => self.op_dup(11)?,
            Operation::Dup13 => self.op_dup(13)?,
            Operation::Dup15 => self.op_dup(15)?,

            Operation::Swap => self.op_swap()?,
            Operation::SwapW => self.op_swapw()?,
            Operation::SwapW2 => self.op_swapw2()?,
            Operation::SwapW3 => self.op_swapw3()?,
            Operation::SwapDW => self.op_swapdw()?,

            Operation::MovUp2 => self.op_movup(2)?,
            Operation::MovUp3 => self.op_movup(3)?,
            Operation::MovUp4 => self.op_movup(4)?,
            Operation::MovUp5 => self.op_movup(5)?,
            Operation::MovUp6 => self.op_movup(6)?,
            Operation::MovUp7 => self.op_movup(7)?,
            Operation::MovUp8 => self.op_movup(8)?,

            Operation::MovDn2 => self.op_movdn(2)?,
            Operation::MovDn3 => self.op_movdn(3)?,
            Operation::MovDn4 => self.op_movdn(4)?,
            Operation::MovDn5 => self.op_movdn(5)?,
            Operation::MovDn6 => self.op_movdn(6)?,
            Operation::MovDn7 => self.op_movdn(7)?,
            Operation::MovDn8 => self.op_movdn(8)?,

            Operation::CSwap => self.op_cswap()?,
            Operation::CSwapW => self.op_cswapw()?,

            // ----- input / output ---------------------------------------------------------------
            Operation::Push(value) => self.op_push(value)?,

            Operation::Read => self.op_read()?,
            Operation::ReadW => self.op_readw()?,

            Operation::MLoadW => self.op_mloadw()?,
            Operation::MStoreW => self.op_mstorew()?,

            Operation::MLoad => self.op_mload()?,
            Operation::MStore => self.op_mstore()?,

            Operation::FmpAdd => self.op_fmpadd()?,
            Operation::FmpUpdate => self.op_fmpupdate()?,

            Operation::SDepth => self.op_sdepth()?,

            // ----- cryptographic operations -----------------------------------------------------
            Operation::RpPerm => self.op_rpperm()?,
            Operation::MpVerify => self.op_mpverify()?,
            Operation::MrUpdate(copy) => self.op_mrupdate(copy)?,
        }

        self.advance_clock();

        Ok(())
    }

    /// Increments the clock cycle for all components of the process.
    fn advance_clock(&mut self) {
        self.system.advance_clock();
        self.stack.advance_clock();
        self.chiplets.advance_clock();
        self.advice.advance_clock();
    }

    /// Makes sure there is enough memory allocated for the trace to accommodate a new clock cycle.
    fn ensure_trace_capacity(&mut self) {
        self.system.ensure_trace_capacity();
        self.stack.ensure_trace_capacity();
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------

    /// Instantiates a new blank process for testing purposes.
    #[cfg(test)]
    fn new_dummy() -> Self {
        Self::new(super::ProgramInputs::none())
    }

    /// Instantiates a new process with an advice tape for testing purposes.
    #[cfg(test)]
    fn new_dummy_with_advice_tape(advice_tape: &[u64]) -> Self {
        let inputs = super::ProgramInputs::new(&[], advice_tape, vec![]).unwrap();
        Self::new(inputs)
    }

    /// Instantiates a new blank process with one decoder trace row for testing purposes. This
    /// allows for setting helpers in the decoder when executing operations during tests.
    #[cfg(test)]
    fn new_dummy_with_decoder_helpers() -> Self {
        let mut process = Self::new(super::ProgramInputs::none());
        process.decoder.add_dummy_trace_row();
        process
    }
}

// TEST HELPERS
// ================================================================================================

/// Pushes proved values onto the stack of the specified process. The values are pushed in the
/// order in which they are provided.
#[cfg(test)]
fn init_stack_with(process: &mut Process, values: &[u64]) {
    let mut result = Vec::with_capacity(values.len());
    for value in values.iter().map(|&v| Felt::new(v)) {
        process.execute_op(Operation::Push(value)).unwrap();
        result.push(value);
    }
}
