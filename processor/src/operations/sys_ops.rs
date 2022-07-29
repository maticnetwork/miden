use super::{
    super::system::{FMP_MAX, FMP_MIN},
    ExecutionError, Felt, FieldElement, Process, StarkField,
};

// SYSTEM OPERATIONS
// ================================================================================================

impl Process {
    /// Pops a value off the stack and asserts that it is equal to ONE.
    ///
    /// # Errors
    /// Returns an error if the popped value is not ONE.
    pub(super) fn op_assert(&mut self) -> Result<(), ExecutionError> {
        if self.stack.get(0) != Felt::ONE {
            return Err(ExecutionError::FailedAssertion(self.system.clk()));
        }
        self.stack.shift_left(1);
        Ok(())
    }

    // FREE MEMORY POINTER
    // --------------------------------------------------------------------------------------------

    /// Pops an element off the stack, adds the current value of the `fmp` register to it, and
    /// pushes the result back onto the stack.
    pub(super) fn op_fmpadd(&mut self) -> Result<(), ExecutionError> {
        let offset = self.stack.get(0);
        let fmp = self.system.fmp();

        self.stack.set(0, fmp + offset);
        self.stack.copy_state(1);

        Ok(())
    }

    /// Pops an element off the stack and adds it to the current value of `fmp` register.
    ///
    /// # Errors
    /// Returns an error if the new value of `fmp` register is greater than or equal to 2^32.
    pub(super) fn op_fmpupdate(&mut self) -> Result<(), ExecutionError> {
        let offset = self.stack.get(0);
        let fmp = self.system.fmp();

        let new_fmp = fmp + offset;
        if new_fmp.as_int() < FMP_MIN || new_fmp.as_int() > FMP_MAX {
            return Err(ExecutionError::InvalidFmpValue(fmp, new_fmp));
        }

        self.system.set_fmp(new_fmp);
        self.stack.shift_left(1);

        Ok(())
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{
        super::{init_stack_with, Operation},
        Felt, FieldElement, Process, FMP_MAX, FMP_MIN,
    };

    #[test]
    fn op_assert() {
        // calling assert with a minimum stack should be an ok, as long as the top value is ONE
        let mut process = Process::new_dummy();
        process.execute_op(Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(Operation::Swap).unwrap();
        process.execute_op(Operation::Drop).unwrap();

        assert!(process.execute_op(Operation::Assert).is_ok());
    }

    #[test]
    fn op_fmpupdate() {
        let mut process = Process::new_dummy();

        // initial value of fmp register should be 2^30
        assert_eq!(Felt::new(2_u64.pow(30)), process.system.fmp());

        // increment fmp register
        process.execute_op(Operation::Push(Felt::new(2))).unwrap();
        process.execute_op(Operation::FmpUpdate).unwrap();
        assert_eq!(Felt::new(FMP_MIN + 2), process.system.fmp());

        // increment fmp register again
        process.execute_op(Operation::Push(Felt::new(3))).unwrap();
        process.execute_op(Operation::FmpUpdate).unwrap();
        assert_eq!(Felt::new(FMP_MIN + 5), process.system.fmp());

        // decrement fmp register
        process.execute_op(Operation::Push(-Felt::new(3))).unwrap();
        process.execute_op(Operation::FmpUpdate).unwrap();
        assert_eq!(Felt::new(FMP_MIN + 2), process.system.fmp());

        // decrementing beyond the minimum fmp value should be an error
        process.execute_op(Operation::Push(-Felt::new(3))).unwrap();
        assert!(process.execute_op(Operation::FmpUpdate).is_err());

        // going up to the max fmp value should be OK
        let mut process = Process::new_dummy();
        process
            .execute_op(Operation::Push(Felt::new(u32::MAX as u64)))
            .unwrap();
        process.execute_op(Operation::FmpUpdate).unwrap();
        assert_eq!(Felt::new(FMP_MAX), process.system.fmp());

        // but going beyond that should be an error
        let mut process = Process::new_dummy();
        process
            .execute_op(Operation::Push(Felt::new(u32::MAX as u64 + 1)))
            .unwrap();
        assert!(process.execute_op(Operation::FmpUpdate).is_err());

        // should not affect the rest of the stack state
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 3]);
        process.execute_op(Operation::FmpUpdate).unwrap();

        let expected = build_expected(&[2]);
        assert_eq!(expected, process.stack.trace_state());

        // calling fmpupdate with a minimum stack should be ok
        let mut process = Process::new_dummy();
        assert!(process.execute_op(Operation::FmpUpdate).is_ok());
    }

    #[test]
    fn op_fmpadd() {
        let mut process = Process::new_dummy();

        // set value of fmp register
        process.execute_op(Operation::Push(Felt::new(2))).unwrap();
        process.execute_op(Operation::FmpUpdate).unwrap();

        // compute address of the first local
        process.execute_op(Operation::Push(-Felt::new(1))).unwrap();
        process.execute_op(Operation::FmpAdd).unwrap();

        let expected = build_expected(&[FMP_MIN + 1]);
        assert_eq!(expected, process.stack.trace_state());

        // compute address of second local (also make sure that rest of stack is not affected)
        process.execute_op(Operation::Push(-Felt::new(2))).unwrap();
        process.execute_op(Operation::FmpAdd).unwrap();

        let expected = build_expected(&[FMP_MIN, FMP_MIN + 1]);
        assert_eq!(expected, process.stack.trace_state());
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    fn build_expected(values: &[u64]) -> [Felt; 16] {
        let mut expected = [Felt::ZERO; 16];
        for (&value, result) in values.iter().zip(expected.iter_mut()) {
            *result = Felt::new(value);
        }
        expected
    }
}
