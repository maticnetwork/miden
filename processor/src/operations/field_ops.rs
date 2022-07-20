use vm_core::StarkField;

use super::{utils::assert_binary, ExecutionError, Felt, FieldElement, Process};

// FIELD OPERATIONS
// ================================================================================================

impl Process {
    // ARITHMETIC OPERATIONS
    // --------------------------------------------------------------------------------------------
    /// Pops two elements off the stack, adds them together, and pushes the result back onto the
    /// stack.
    pub(super) fn op_add(&mut self) -> Result<(), ExecutionError> {
        let b = self.stack.get(0);
        let a = self.stack.get(1);
        self.stack.set(0, a + b);
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops an element off the stack, computes its additive inverse, and pushes the result back
    /// onto the stack.
    pub(super) fn op_neg(&mut self) -> Result<(), ExecutionError> {
        let a = self.stack.get(0);
        self.stack.set(0, -a);
        self.stack.copy_state(1);
        Ok(())
    }

    /// Pops two elements off the stack, multiplies them, and pushes the result back onto the
    /// stack.
    pub(super) fn op_mul(&mut self) -> Result<(), ExecutionError> {
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
    /// Returns an error if the value on the top of the stack is ZERO.
    pub(super) fn op_inv(&mut self) -> Result<(), ExecutionError> {
        let a = self.stack.get(0);
        if a == Felt::ZERO {
            return Err(ExecutionError::DivideByZero(self.system.clk()));
        }

        self.stack.set(0, a.inv());
        self.stack.copy_state(1);
        Ok(())
    }

    /// Pops an element off the stack, adds ONE to it, and pushes the result back onto the stack.
    pub(super) fn op_incr(&mut self) -> Result<(), ExecutionError> {
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
    /// Returns an error if either of the two elements on the top of the stack is not a binary
    /// value.
    pub(super) fn op_and(&mut self) -> Result<(), ExecutionError> {
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
    /// Returns an error if either of the two elements on the top of the stack is not a binary
    /// value.
    pub(super) fn op_or(&mut self) -> Result<(), ExecutionError> {
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
    /// Returns an error if the value on the top of the stack is not a binary value.
    pub(super) fn op_not(&mut self) -> Result<(), ExecutionError> {
        let a = assert_binary(self.stack.get(0))?;
        self.stack.set(0, Felt::ONE - a);
        self.stack.copy_state(1);
        Ok(())
    }

    // COMPARISON OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Pops two elements off the stack and compares them. If the elements are equal, pushes ONE
    /// onto the stack, otherwise pushes ZERO onto the stack.
    pub(super) fn op_eq(&mut self) -> Result<(), ExecutionError> {
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
    pub(super) fn op_eqz(&mut self) -> Result<(), ExecutionError> {
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
    pub(super) fn op_eqw(&mut self) -> Result<(), ExecutionError> {
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

    /// Computes a single turn of binary accumulation for the given inputs. The stack is arranged
    /// as follows (from the top):
    /// - exponent of 2 for this turn - 1 element
    /// - accumulated power of 2 so far - 1 element
    /// - number which needs to be shifted to the right - 1 element
    ///
    /// To perform the operation we do the following:
    /// 1. Pops top three elements off the stack and calculate the least significant bit of the
    /// number `b`.
    /// 2. Use this bit to decide if the current 2 raise to the power exponent needs to be included
    /// in the accumulator.
    /// 3. Update exponent with its square and the number b with one right shift.
    /// 4. Pushes the calcuted new values to the stack in the mentioned order.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Exponent is not a power of 2.
    /// - Accumulator is not a power of 2.
    pub(super) fn op_binacc(&mut self) -> Result<(), ExecutionError> {
        let mut exp = self.stack.get(0);
        let mut acc = self.stack.get(1);
        let mut b = self.stack.get(2);

        // Both exp and acc should be a power of 2. Infact log of exp should also be a power
        // of 2 as well.
        if !exp.as_int().is_power_of_two() {
            return Err(ExecutionError::InvalidPowerOfTwo(exp));
        }
        if !acc.as_int().is_power_of_two() {
            return Err(ExecutionError::InvalidPowerOfTwo(exp));
        }

        // least significant bit of the number `b`.
        let bit = b.as_int() & 1;

        // current value of 2 raise to the power `exponent` added to the accumulator.
        acc *= Felt::new((exp.as_int() - 1) * bit + 1);

        // number `b` shifted right by one bit.
        b = Felt::new(b.as_int() >> 1);

        // exponent updated with its square.
        exp *= exp;

        self.stack.set(0, exp);
        self.stack.set(1, acc);
        self.stack.set(2, b);
        self.stack.copy_state(3);
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
    use vm_core::MIN_STACK_DEPTH;

    // ARITHMETIC OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_add() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // add the top two values
        process.execute_op(Operation::Add).unwrap();
        let expected = build_expected(&[a + b, c]);

        assert_eq!(MIN_STACK_DEPTH + 2, process.stack.depth());
        assert_eq!(4, process.stack.current_clk());
        assert_eq!(expected, process.stack.trace_state());

        // calling add with a stack of minimum depth is ok
        let mut process = Process::new_dummy();
        assert!(process.execute_op(Operation::Add).is_ok());
    }

    #[test]
    fn op_neg() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // negate the top value
        process.execute_op(Operation::Neg).unwrap();
        let expected = build_expected(&[-a, b, c]);

        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(MIN_STACK_DEPTH + 3, process.stack.depth());
        assert_eq!(4, process.stack.current_clk());
    }

    #[test]
    fn op_mul() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // add the top two values
        process.execute_op(Operation::Mul).unwrap();
        let expected = build_expected(&[a * b, c]);

        assert_eq!(MIN_STACK_DEPTH + 2, process.stack.depth());
        assert_eq!(4, process.stack.current_clk());
        assert_eq!(expected, process.stack.trace_state());

        // calling mul with a stack of minimum depth is ok
        let mut process = Process::new_dummy();
        assert!(process.execute_op(Operation::Mul).is_ok());
    }

    #[test]
    fn op_inv() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // invert the top value
        if b != Felt::ZERO {
            process.execute_op(Operation::Inv).unwrap();
            let expected = build_expected(&[a.inv(), b, c]);

            assert_eq!(MIN_STACK_DEPTH + 3, process.stack.depth());
            assert_eq!(4, process.stack.current_clk());
            assert_eq!(expected, process.stack.trace_state());
        }

        // inverting zero should be an error
        process.execute_op(Operation::Pad).unwrap();
        assert!(process.execute_op(Operation::Inv).is_err());
    }

    #[test]
    fn op_incr() {
        // initialize the stack with a few values
        let mut process = Process::new_dummy();
        let (a, b, c) = init_stack_rand(&mut process);

        // negate the top value
        process.execute_op(Operation::Incr).unwrap();
        let expected = build_expected(&[a + Felt::ONE, b, c]);

        assert_eq!(MIN_STACK_DEPTH + 3, process.stack.depth());
        assert_eq!(4, process.stack.current_clk());
        assert_eq!(expected, process.stack.trace_state());
    }

    // BOOLEAN OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_and() {
        // --- test 0 AND 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0, 0]);

        process.execute_op(Operation::And).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 1 AND 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0, 1]);

        process.execute_op(Operation::And).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 0 AND 1 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 0]);

        process.execute_op(Operation::And).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 1 AND 1 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 1]);

        process.execute_op(Operation::And).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- first operand is not binary ------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 2]);
        assert!(process.execute_op(Operation::And).is_err());

        // --- second operand is not binary -----------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 2, 1]);
        assert!(process.execute_op(Operation::And).is_err());

        // --- calling AND with a stack of minimum depth is ok ----------------
        let mut process = Process::new_dummy();
        assert!(process.execute_op(Operation::And).is_ok());
    }

    #[test]
    fn op_or() {
        // --- test 0 OR 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0, 0]);

        process.execute_op(Operation::Or).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 1 OR 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0, 1]);

        process.execute_op(Operation::Or).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 0 OR 1 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 0]);

        process.execute_op(Operation::Or).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test 1 OR 0 ---------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 1]);

        process.execute_op(Operation::Or).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- first operand is not binary ------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1, 2]);
        assert!(process.execute_op(Operation::Or).is_err());

        // --- second operand is not binary -----------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 2, 1]);
        assert!(process.execute_op(Operation::Or).is_err());

        // --- calling OR with a stack of minimum depth is a ok ----------------
        let mut process = Process::new_dummy();
        assert!(process.execute_op(Operation::Or).is_ok());
    }

    #[test]
    fn op_not() {
        // --- test NOT 0 -----------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 0]);

        process.execute_op(Operation::Not).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test NOT 1 ----------------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 1]);

        process.execute_op(Operation::Not).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(2)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- operand is not binary ------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[2, 2]);
        assert!(process.execute_op(Operation::Not).is_err());
    }

    // COMPARISON OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_eq() {
        // --- test when top two values are equal -----------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 7, 7]);

        process.execute_op(Operation::Eq).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when top two values are not equal -------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 5, 7]);

        process.execute_op(Operation::Eq).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- calling EQ with a stack of minimum depth is a ok ---------------
        let mut process = Process::new_dummy();
        assert!(process.execute_op(Operation::Eq).is_ok());
    }

    #[test]
    fn op_eqz() {
        // --- test when top is zero ------------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 0]);

        process.execute_op(Operation::Eqz).unwrap();
        let expected = build_expected(&[Felt::ONE, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when top is not zero --------------------------------------
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[3, 4]);

        process.execute_op(Operation::Eqz).unwrap();
        let expected = build_expected(&[Felt::ZERO, Felt::new(3)]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_eqw() {
        // --- test when top two words are equal ------------------------------
        let mut process = Process::new_dummy();
        let mut values = vec![1, 2, 3, 4, 5, 2, 3, 4, 5];
        init_stack_with(&mut process, &values);

        process.execute_op(Operation::Eqw).unwrap();
        values.reverse();
        values.insert(0, 1);
        let expected = build_expected_from_ints(&values);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when top two words are not equal --------------------------
        let mut process = Process::new_dummy();
        let mut values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        init_stack_with(&mut process, &values);

        process.execute_op(Operation::Eqw).unwrap();
        values.reverse();
        values.insert(0, 0);
        let expected = build_expected_from_ints(&values);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_binacc() {
        // --- test when b become 0 -------------------------------------------------------------------------------
        let mut process = Process::new_dummy();
        let a = 0;
        let b = 32;
        let c = 4;
        init_stack_with(&mut process, &[a, b, c]);

        process.execute_op(Operation::BinAcc).unwrap();
        let expected = build_expected(&[Felt::new(16), Felt::new(32), Felt::new(a >> 1)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when bit from b is 1 ---------------------------------------------------------------------------
        let mut process = Process::new_dummy();
        let a = 3;
        let b = 1;
        let c = 16;
        init_stack_with(&mut process, &[a, b, c]);

        process.execute_op(Operation::BinAcc).unwrap();
        let expected = build_expected(&[Felt::new(256), Felt::new(16), Felt::new(a >> 1)]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test when bit from b is 1 & exp is 2**32. exp will overflow the field after this operation -----------
        let mut process = Process::new_dummy();
        let a = 1;
        let b = 16;
        let c = 2u64.pow(32);
        init_stack_with(&mut process, &[a, b, c]);

        process.execute_op(Operation::BinAcc).unwrap();
        let expected = build_expected(&[
            Felt::new(2u64.pow(32)) * Felt::new(2u64.pow(32)),
            Felt::new(2u64.pow(36)),
            Felt::new(a >> 1),
        ]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test fails, exp not pow of 2 ----------------------------------------------------------------------------
        let mut process = Process::new_dummy();
        let a = 3;
        let b = 1;
        let c = 15;
        init_stack_with(&mut process, &[a, b, c]);
        assert!(process.execute_op(Operation::BinAcc).is_err());

        // --- test fails, acc not pow of 2 ----------------------------------------------------------------------------
        let mut process = Process::new_dummy();
        let a = 2;
        let b = 45;
        let c = 4;
        init_stack_with(&mut process, &[a, b, c]);
        assert!(process.execute_op(Operation::BinAcc).is_err());

        // --- test fails, both acc & exp not pow of 2 ------------------------------------------------------------------
        let mut process = Process::new_dummy();
        let a = 2;
        let b = 47;
        let c = 5;
        init_stack_with(&mut process, &[a, b, c]);
        assert!(process.execute_op(Operation::BinAcc).is_err());
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
