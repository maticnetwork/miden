use super::{utils::assert_binary, ExecutionError, Felt, Process, StarkField};

impl Process {
    // CASTING OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Pops the top element off the stack, splits it into low and high 32-bit values, and pushes
    /// these values back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack is empty.
    pub(super) fn op_u32split(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(1, "U32SPLIT")?;

        let a = self.stack.get(0);
        let (lo, hi) = split_element(a);

        // shift right first so stack depth is increased before we attempt to set the output values
        self.stack.shift_right(1);
        self.stack.set(0, hi);
        self.stack.set(1, lo);
        Ok(())
    }

    // ARITHMETIC OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Pops two elements off the stack, adds them, splits the result into low and high 32-bit
    /// values, and pushes these values back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_u32add(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "U32ADD")?;

        let b = self.stack.get(0);
        let a = self.stack.get(1);
        let result = a + b;
        let (lo, hi) = split_element(result);

        self.stack.set(0, hi);
        self.stack.set(1, lo);
        self.stack.copy_state(2);
        Ok(())
    }

    /// Pops three elements off the stack, adds them, splits the result into low and high 32-bit
    /// values, and pushes these values back onto the stack.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The stack contains fewer than three elements.
    /// * The third element from the top fo the stack is not a binary value.
    pub(super) fn op_u32addc(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(3, "U32ADDC")?;

        let b = self.stack.get(0).as_int();
        let a = self.stack.get(1).as_int();
        let c = assert_binary(self.stack.get(2))?.as_int();
        let result = Felt::new(a + b + c);
        let (lo, hi) = split_element(result);

        self.stack.set(0, hi);
        self.stack.set(1, lo);
        self.stack.shift_left(3);
        Ok(())
    }

    /// Pops two elements off the stack, subtracts the top element from the second element, and
    /// pushes the result as well as a flag indicating whether there was underflow back onto the
    /// stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_u32sub(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "U32SUB")?;

        let b = self.stack.get(0).as_int();
        let a = self.stack.get(1).as_int();
        let result = a.wrapping_sub(b);

        self.stack.set(0, Felt::new(result >> 63));
        self.stack.set(1, Felt::new((result as u32) as u64));
        self.stack.copy_state(2);
        Ok(())
    }

    /// Pops two elements off the stack, multiplies them, splits the result into low and high
    /// 32-bit values, and pushes these values back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_u32mul(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "U32MUL")?;

        let b = self.stack.get(0).as_int();
        let a = self.stack.get(1).as_int();
        let result = Felt::new(a * b);
        let (lo, hi) = split_element(result);

        self.stack.set(0, hi);
        self.stack.set(1, lo);
        self.stack.copy_state(2);
        Ok(())
    }

    /// Pops three elements off the stack, multiplies the first two and adds the third element to
    /// the result, splits the result into low and high 32-bit values, and pushes these values
    /// back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than three elements.
    pub(super) fn op_u32madd(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(3, "U32MADD")?;

        let b = self.stack.get(0).as_int();
        let a = self.stack.get(1).as_int();
        let c = self.stack.get(2).as_int();
        let result = Felt::new(a * b + c);
        let (lo, hi) = split_element(result);

        self.stack.set(0, hi);
        self.stack.set(1, lo);
        self.stack.shift_left(3);
        Ok(())
    }

    /// Pops two elements off the stack, divides the second element by the top element, and pushes
    /// the quotient and the remainder back onto the stack.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The stack contains fewer than two elements.
    /// * The divisor is ZERO.
    pub(super) fn op_u32div(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "U32DIV")?;

        let b = self.stack.get(0).as_int();
        let a = self.stack.get(1).as_int();

        if b == 0 {
            return Err(ExecutionError::DivideByZero(self.system.clk()));
        }

        let q = a / b;
        let r = a - q * b;

        self.stack.set(0, Felt::new(r));
        self.stack.set(1, Felt::new(q));
        self.stack.copy_state(2);
        Ok(())
    }

    // BITWISE OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Pops two elements off the stack, computes their bitwise AND, splits the result into low and
    /// high 32-bit values, and pushes these values back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_u32and(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "U32AND")?;

        let b = self.stack.get(0);
        let a = self.stack.get(1);
        let result = self.bitwise.u32and(a, b)?;

        self.stack.set(0, result);
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops two elements off the stack, computes their bitwise OR, splits the result into low and
    /// high 32-bit values, and pushes these values back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_u32or(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "U32OR")?;

        let b = self.stack.get(0);
        let a = self.stack.get(1);
        let result = self.bitwise.u32or(a, b)?;

        self.stack.set(0, result);
        self.stack.shift_left(2);
        Ok(())
    }

    /// Pops two elements off the stack, computes their bitwise XOR, splits the result into low and
    /// high 32-bit values, and pushes these values back onto the stack.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than two elements.
    pub(super) fn op_u32xor(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(2, "U32XOR")?;

        let b = self.stack.get(0);
        let a = self.stack.get(1);
        let result = self.bitwise.u32xor(a, b)?;

        self.stack.set(0, result);
        self.stack.shift_left(2);
        Ok(())
    }
}

// HELPER FUNCTIONS
// ================================================================================================

#[inline(always)]
fn split_element(value: Felt) -> (Felt, Felt) {
    let value = value.as_int();
    let lo = (value as u32) as u64;
    let hi = value >> 32;
    (Felt::new(lo), Felt::new(hi))
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

    // CASTING OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_u32split() {
        // --- test a random value ---------------------------------------------
        let a: u64 = rand_value();
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[a]);
        let hi = a >> 32;
        let lo = (a as u32) as u64;

        process.execute_op(&Operation::U32split).unwrap();
        let mut expected = [Felt::ZERO; 16];
        expected[0] = Felt::new(hi);
        expected[1] = Felt::new(lo);
        assert_eq!(expected, process.stack.trace_state());

        // --- test the rest of the stack is not modified -----------------------
        let b: u64 = rand_value();
        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[a, b]);
        let hi = b >> 32;
        let lo = (b as u32) as u64;

        process.execute_op(&Operation::U32split).unwrap();
        let mut expected = [Felt::ZERO; 16];
        expected[0] = Felt::new(hi);
        expected[1] = Felt::new(lo);
        expected[2] = Felt::new(a);
        assert_eq!(expected, process.stack.trace_state());
    }

    // ARITHMETIC OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_u32add() {
        // --- test random values ---------------------------------------------
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);
        let (result, over) = a.overflowing_add(b);

        process.execute_op(&Operation::U32add).unwrap();
        let expected = build_expected(&[over as u32, result, c, d]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test overflow --------------------------------------------------
        let a = u32::MAX - 1;
        let b = 2u32;

        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[a as u64, b as u64]);
        let (result, over) = a.overflowing_add(b);

        process.execute_op(&Operation::U32add).unwrap();
        let expected = build_expected(&[over as u32, result]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_u32addc() {
        // --- test c = 1 -----------------------------------------------------
        let a = (rand_value::<u64>() as u32) as u64;
        let b = (rand_value::<u64>() as u32) as u64;
        let c = 1u64;
        let d = (rand_value::<u64>() as u32) as u64;

        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[d, c, b, a]);

        let result = a + b + c;
        let hi = (result >> 32) as u32;
        let lo = result as u32;
        assert!(hi <= 1);

        process.execute_op(&Operation::U32addc).unwrap();
        let expected = build_expected(&[hi, lo, d as u32]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test c > 1 -----------------------------------------------------
        let a = (rand_value::<u64>() as u32) as u64;
        let b = (rand_value::<u64>() as u32) as u64;
        let c = 2u64;

        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[c, b, a]);
        assert!(process.execute_op(&Operation::U32addc).is_err());
    }

    #[test]
    fn op_u32sub() {
        // --- test random values ---------------------------------------------
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);
        let (result, under) = b.overflowing_sub(a);

        process.execute_op(&Operation::U32sub).unwrap();
        let expected = build_expected(&[under as u32, result, c, d]);
        assert_eq!(expected, process.stack.trace_state());

        // --- test underflow -------------------------------------------------
        let a = 10u32;
        let b = 11u32;

        let mut process = Process::new_dummy();
        init_stack_with(&mut process, &[a as u64, b as u64]);
        let (result, under) = a.overflowing_sub(b);

        process.execute_op(&Operation::U32sub).unwrap();
        let expected = build_expected(&[under as u32, result]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_u32mul() {
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);
        let result = (a as u64) * (b as u64);
        let hi = (result >> 32) as u32;
        let lo = result as u32;

        process.execute_op(&Operation::U32mul).unwrap();
        let expected = build_expected(&[hi, lo, c, d]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_u32madd() {
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);
        let result = (a as u64) * (b as u64) + (c as u64);
        let hi = (result >> 32) as u32;
        let lo = result as u32;

        process.execute_op(&Operation::U32madd).unwrap();
        let expected = build_expected(&[hi, lo, d]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_u32div() {
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);
        let q = b / a;
        let r = b % a;

        process.execute_op(&Operation::U32div).unwrap();
        let expected = build_expected(&[r, q, c, d]);
        assert_eq!(expected, process.stack.trace_state());
    }

    // BITWISE OPERATIONS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_u32and() {
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);

        process.execute_op(&Operation::U32and).unwrap();
        let expected = build_expected(&[a & b, c, d]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_u32or() {
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);

        process.execute_op(&Operation::U32or).unwrap();
        let expected = build_expected(&[a | b, c, d]);
        assert_eq!(expected, process.stack.trace_state());
    }

    #[test]
    fn op_u32xor() {
        let mut process = Process::new_dummy();
        let (a, b, c, d) = init_stack_rand(&mut process);

        process.execute_op(&Operation::U32xor).unwrap();
        let expected = build_expected(&[a ^ b, c, d]);
        assert_eq!(expected, process.stack.trace_state());
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    fn init_stack_rand(process: &mut Process) -> (u32, u32, u32, u32) {
        // push values a and b onto the stack
        let a = rand_value::<u64>() as u32;
        let b = rand_value::<u64>() as u32;
        let c = rand_value::<u64>() as u32;
        let d = rand_value::<u64>() as u32;
        init_stack_with(process, &[a as u64, b as u64, c as u64, d as u64]);
        (d, c, b, a)
    }

    fn build_expected(values: &[u32]) -> [Felt; 16] {
        let mut expected = [Felt::ZERO; 16];
        for (&value, result) in values.iter().zip(expected.iter_mut()) {
            *result = Felt::new(value as u64);
        }
        expected
    }
}
