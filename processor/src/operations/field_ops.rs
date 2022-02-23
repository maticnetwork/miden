use super::{utils::assert_binary, ExecutionError, Felt, FieldElement, Process};

// FIELD OPERATIONS
// ================================================================================================

impl Process {
    // ARITHMETIC OPERATIONS
    // --------------------------------------------------------------------------------------------
    /// Pops two elements off the stack, adds them together, and pushes the result back onto the
    /// stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_add(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "ADD")?;

        let b = self.stack.get(0);
        let a = self.stack.get(1);
        self.stack.set(0, a + b);
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops an element off the stack, computes its additive inverse, and pushes the result back
    /// onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack is empty.
    pub(super) fn op_neg(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(1, "NEG")?;

        let a = self.stack.get(0);
        self.stack.set(0, -a);
        self.stack.copy_state(1);
        Ok(())
    }

    /// Pops two elements off the stack, multiplies them, and pushes the result back onto the
    /// stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_mul(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "MUL")?;

        let b = self.stack.get(0);
        let a = self.stack.get(1);
        self.stack.set(0, a * b);
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops an element off the stack, computes its multiplicative inverse, and pushes the result
    /// back onto the stack.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The stack is empty.
    /// * The value on the top of the stack is ZERO.
    pub(super) fn op_inv(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(1, "INV")?;

        let a = self.stack.get(0);
        if a == Felt::ZERO {
            return Err(ExecutionError::DivideByZero(self.system.clk()));
        }

        self.stack.set(0, a.inv());
        self.stack.copy_state(1);
        Ok(())
    }

    /// Pops an element off the stack, adds ONE to it, and pushes the result back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack is empty.
    pub(super) fn op_incr(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(1, "INCR")?;

        let a = self.stack.get(0);
        self.stack.set(0, a + Felt::ONE);
        self.stack.copy_state(1);
        Ok(())
    }

    // BOOLEAN OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Pops two elements off the stack, computes their boolean AND, and pushes the result back
    /// onto the stack.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The stack contains fewer than two elements.
    /// * Either of the two elements on the top of the stack is not a binary value.
    pub(super) fn op_and(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "AND")?;

        let b = assert_binary(self.stack.get(0))?;
        let a = assert_binary(self.stack.get(1))?;
        if a == Felt::ONE && b == Felt::ONE {
            self.stack.set(0, Felt::ONE);
        } else {
            self.stack.set(0, Felt::ZERO);
        }
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops two elements off the stack, computes their boolean OR, and pushes the result back
    /// onto the stack.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The stack contains fewer than two elements.
    /// * Either of the two elements on the top of the stack is not a binary value.
    pub(super) fn op_or(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "OR")?;

        let b = assert_binary(self.stack.get(0))?;
        let a = assert_binary(self.stack.get(1))?;
        if a == Felt::ONE || b == Felt::ONE {
            self.stack.set(0, Felt::ONE);
        } else {
            self.stack.set(0, Felt::ZERO);
        }
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops an element off the stack, computes its boolean NOT, and pushes the result back onto
    /// the stack.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The stack is empty.
    /// * The value on the top of the stack is not a binary value.
    pub(super) fn op_not(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(1, "NOT")?;

        let a = assert_binary(self.stack.get(0))?;
        self.stack.set(0, Felt::ONE - a);
        self.stack.copy_state(1);
        Ok(())
    }

    // COMPARISON OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Pops two elements off the stack and compares them. If the elements are equal, pushes ONE
    /// onto the stack, otherwise pushes ZERO onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_eq(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "EQ")?;

        let b = self.stack.get(0);
        let a = self.stack.get(1);
        if a == b {
            self.stack.set(0, Felt::ONE);
        } else {
            self.stack.set(0, Felt::ZERO);
        }
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops an element off the stack and compares it to ZERO. If the element is ZERO, pushes ONE
    /// onto the stack, otherwise pushes ZERO onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack is empty.
    pub(super) fn op_eqz(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(1, "EQZ")?;

        let a = self.stack.get(0);
        if a == Felt::ZERO {
            self.stack.set(0, Felt::ONE);
        } else {
            self.stack.set(0, Felt::ZERO);
        }
        self.stack.copy_state(1);
        Ok(())
    }

    /// Compares the first word (four elements) with the second word on the stack, if the words are
    /// equal, pushes ONE onto the stack, otherwise pushes ZERO onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than 8 elements.
    pub(super) fn op_eqw(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(8, "EQW")?;

        let b3 = self.stack.get(0);
        let b2 = self.stack.get(1);
        let b1 = self.stack.get(2);
        let b0 = self.stack.get(3);

        let a3 = self.stack.get(4);
        let a2 = self.stack.get(5);
        let a1 = self.stack.get(6);
        let a0 = self.stack.get(7);

        if a0 == b0 && a1 == b1 && a2 == b2 && a3 == b3 {
            self.stack.set(0, Felt::ONE);
        } else {
            self.stack.set(0, Felt::ZERO);
        }
        self.stack.shift_right(0);
        Ok(())
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{
        super::{init_stack_with, Felt, FieldElement, Operation},
        Process,
    };
    use rand_utils::rand_value;

    // ARITHMETIC OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_add() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // add the top two values
        process.execute_op(&Operation::Add).unwrap();
        let expected = build_expected(&[a + b, c]);

        assert_eq!(2, process.stack.depth());
        assert_eq!(4, process.stack.current_step());
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_neg() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // negate the top value
        process.execute_op(&Operation::Neg).unwrap();
        let expected = build_expected(&[-a, b, c]);

        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(3, process.stack.depth());
        assert_eq!(4, process.stack.current_step());
    }

    #[test]
    fn op_mul() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // add the top two values
        process.execute_op(&Operation::Mul).unwrap();
        let expected = build_expected(&[a * b, c]);

        assert_eq!(2, process.stack.depth());
        assert_eq!(4, process.stack.current_step());
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_inv() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // invert the top value
        if b != Felt::ZERO {
            process.execute_op(&Operation::Inv).unwrap();
            let expected = build_expected(&[a.inv(), b, c]);

            assert_eq!(3, process.stack.depth());
            assert_eq!(4, process.stack.current_step());
            assert_eq!(expected, process.stack.trace_state());
        }

        // inverting zero should be an error
        process.execute_op(&Operation::Pad).unwrap();
        assert!(process.execute_op(&Operation::Inv).is_err());
    }

    #[test]
    fn op_incr() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // negate the top value
        process.execute_op(&Operation::Incr).unwrap();
        let expected = build_expected(&[a + Felt::ONE, b, c]);

        assert_eq!(3, process.stack.depth());
        assert_eq!(4, process.stack.current_step());
        assert_eq!(expected, process.stack.trace_state());
    }

    // BOOLEAN OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_and() {
        // --- test 0 AND 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0, 0]);

        process.execute_op(&Operation::And).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 1 AND 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0, 1]);

        process.execute_op(&Operation::And).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 0 AND 1 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 0]);

        process.execute_op(&Operation::And).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 1 AND 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 1]);

        process.execute_op(&Operation::And).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- first operand is not binary ------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 2]);
        assert!(process.execute_op(&Operation::And).is_err());

        // --- second operand is not binary ------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 2, 1]);
        assert!(process.execute_op(&Operation::And).is_err());
    }

    #[test]
    fn op_not() {
        // --- test NOT 0 -----------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0]);

        process.execute_op(&Operation::Not).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test NOT 1 ----------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1]);

        process.execute_op(&Operation::Not).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- operand is not binary ------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 2]);
        assert!(process.execute_op(&Operation::Not).is_err());
    }

    // COMPARISON OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_eq() {
        // --- test when top two values are equal -----------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 7, 7]);

        process.execute_op(&Operation::Eq).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when top two values are not equal -------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 5, 7]);

        process.execute_op(&Operation::Eq).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_eqz() {
        // --- test when top is zero ------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 0]);

        process.execute_op(&Operation::Eqz).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when top is not zero --------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 4]);

        process.execute_op(&Operation::Eqz).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_eqw() {
        // --- test when top two words are equal ------------------------------
        let mut process = Process::new_dummy();
        let mut values = vec![1, 2, 3, 4, 5, 2, 3, 4, 5];
        init_stack_with(&mut process, &values);

        process.execute_op(&Operation::Eqw).unwrap();
        values.reverse();
        values.insert(0, 1);
        let expected = build_expected_from_ints(&values);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when top two words are not equal --------------------------
        let mut process = Process::new_dummy();
        let mut values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        init_stack_with(&mut process, &values);

        process.execute_op(&Operation::Eqw).unwrap();
        values.reverse();
        values.insert(0, 0);
        let expected = build_expected_from_ints(&values);
        assert_eq!(expected, process.stack.trace_state());
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    fn init_stack_rand(process: &mut Process) -> (Felt, Felt, Felt) {
        // push values a and b onto the stack
        let a = rand_value();
        let b = rand_value();
        let c = rand_value();
        init_stack_with(process, &[a, b, c]);
        (Felt::new(c), Felt::new(b), Felt::new(a))
    }

    fn build_expected(values: &[Felt]) -> [Felt; 16] {
        let mut expected = [Felt::ZERO; 16];
        for (&value, result) in values.iter().zip(expected.iter_mut()) {
            *result = value;
        }
        expected
    }

    fn build_expected_from_ints(values: &[u64]) -> [Felt; 16] {
        let mut expected = [Felt::ZERO; 16];
        for (&value, result) in values.iter().zip(expected.iter_mut()) {
            *result = Felt::new(value);
        }
        expected
    }
}
