use super::{ExecutionError, Felt, Process};

// INPUT / OUTPUT OPERATIONS
// ================================================================================================

impl Process {
    // CONSTANT INPUTS
    // --------------------------------------------------------------------------------------------

    /// Pushes the provided value onto the stack.
    ///
    /// The original stack is shifted to the right by one item.
    pub(super) fn op_push(&mut self, value: Felt) -> Result<(), ExecutionError> {
        self.stack.set(0, value);
        self.stack.shift_right(0);
        Ok(())
    }

    // MEMORY READING AND WRITING
    // --------------------------------------------------------------------------------------------

    /// Loads a word (4 elements) from the specified memory address onto the stack.
    ///
    /// The operation works as follows:
    /// - The memory address is popped off the stack.
    /// - A word is retrieved from memory at the specified address. The memory is always
    ///   initialized to ZEROs, and thus, if the specified address has never been written to,
    ///   four ZERO elements are returned.
    /// - The top four elements of the stack are overwritten with values retried from memory.
    ///
    /// Thus, the net result of the operation is that the stack is shifted left by one item.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than five elements.
    pub(super) fn op_loadw(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(5, "LOADW")?;

        // get the address from the stack and read the word from memory
        let addr = self.stack.get(0);
        let word = self.memory.read(addr);

        // update the stack state
        for (i, &value) in word.iter().rev().enumerate() {
            self.stack.set(i, value);
        }
        self.stack.shift_left(5);

        Ok(())
    }

    /// Stores a word (4 elements) from the stack into the specified memory address.
    ///
    /// The operation works as follows:
    /// - The memory address is popped off the stack.
    /// - The top four stack items are saved into the specified memory address. The items are not
    ///   removed from the stack.
    ///
    /// Thus, the net result of the operation is that the stack is shifted left by one item.
    ///
    /// # Errors
    /// Returns an error if the stack contains fewer than five elements.
    pub(super) fn op_storew(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(5, "STOREW")?;

        // get the address from the stack and build the word to be saved from the stack values
        let addr = self.stack.get(0);
        let word = [
            self.stack.get(4),
            self.stack.get(3),
            self.stack.get(2),
            self.stack.get(1),
        ];

        // write the word to memory
        self.memory.write(addr, word);

        // update the stack state
        for (i, &value) in word.iter().rev().enumerate() {
            self.stack.set(i, value);
        }
        self.stack.shift_left(5);

        Ok(())
    }

    // ADVICE INPUTS
    // --------------------------------------------------------------------------------------------

    /// Removes the next element from the advice tape and pushes it onto the stack.
    ///
    /// # Errors
    /// Returns an error if the advice tape is empty.
    pub(super) fn op_read(&mut self) -> Result<(), ExecutionError> {
        let value = self.advice.read_tape()?;
        self.stack.set(0, value);
        self.stack.shift_right(0);
        Ok(())
    }

    /// Removes a word (4 elements) from the advice tape and overwrites the top four stack
    /// elements with it.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The stack contains fewer than four elements.
    /// * The advice tape contains fewer than four elements.
    pub(super) fn op_readw(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(4, "READW")?;

        let a = self.advice.read_tape()?;
        let b = self.advice.read_tape()?;
        let c = self.advice.read_tape()?;
        let d = self.advice.read_tape()?;

        self.stack.set(0, d);
        self.stack.set(1, c);
        self.stack.set(2, b);
        self.stack.set(3, a);
        self.stack.copy_state(4);

        Ok(())
    }

    // ENVIRONMENT INPUTS
    // --------------------------------------------------------------------------------------------

    /// Pushes the current depth of the stack (the depth before this operation is executed) onto
    /// the stack.
    pub(super) fn op_sdepth(&mut self) -> Result<(), ExecutionError> {
        let stack_depth = self.stack.depth();
        self.stack.set(0, Felt::new(stack_depth as u64));
        self.stack.shift_right(0);
        Ok(())
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{
        super::{FieldElement, Operation},
        Felt, Process,
    };

    #[test]
    fn op_push() {
        let mut process = Process::new_dummy();
        assert_eq!(0, process.stack.depth());
        assert_eq!(0, process.stack.current_step());
        assert_eq!([Felt::ZERO; 16], process.stack.trace_state());

        // push one item onto the stack
        let op = Operation::Push(Felt::ONE);
        process.execute_op(&op).unwrap();
        let mut expected = [Felt::ZERO; 16];
        expected[0] = Felt::ONE;

        assert_eq!(1, process.stack.depth());
        assert_eq!(1, process.stack.current_step());
        assert_eq!(expected, process.stack.trace_state());

        // push another item onto the stack
        let op = Operation::Push(Felt::new(3));
        process.execute_op(&op).unwrap();
        let mut expected = [Felt::ZERO; 16];
        expected[0] = Felt::new(3);
        expected[1] = Felt::ONE;

        assert_eq!(2, process.stack.depth());
        assert_eq!(2, process.stack.current_step());
        assert_eq!(expected, process.stack.trace_state());
    }

    // MEMORY OPERATION TESTS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_storew() {
        let mut process = Process::new_dummy();
        assert_eq!(0, process.memory.size());

        // push the first word onto the stack and save it at address 0
        let word1 = [Felt::new(1), Felt::new(3), Felt::new(5), Felt::new(7)];
        store_value(&mut process, 0, word1);

        // check stack state
        let expected_stack = build_expected_stack(&[7, 5, 3, 1]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        assert_eq!(1, process.memory.size());
        assert_eq!(word1, process.memory.get_value(0).unwrap());

        // push the second word onto the stack and save it at address 3
        let word2 = [Felt::new(2), Felt::new(4), Felt::new(6), Felt::new(8)];
        store_value(&mut process, 3, word2);

        // check stack state
        let expected_stack = build_expected_stack(&[8, 6, 4, 2, 7, 5, 3, 1]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        assert_eq!(2, process.memory.size());
        assert_eq!(word1, process.memory.get_value(0).unwrap());
        assert_eq!(word2, process.memory.get_value(3).unwrap());
    }

    #[test]
    fn op_loadw() {
        let mut process = Process::new_dummy();
        assert_eq!(0, process.memory.size());

        // push a word onto the stack and save it at address 1
        let word = [Felt::new(1), Felt::new(3), Felt::new(5), Felt::new(7)];
        store_value(&mut process, 1, word);

        // push four zeros onto the stack
        for _ in 0..4 {
            process.execute_op(&Operation::Pad).unwrap();
        }

        // push the address onto the stack and load the word
        process.execute_op(&Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(&Operation::LoadW).unwrap();

        let expected_stack = build_expected_stack(&[7, 5, 3, 1, 7, 5, 3, 1]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        assert_eq!(1, process.memory.size());
        assert_eq!(word, process.memory.get_value(1).unwrap());
    }

    // ADVICE INPUT TESTS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_read() {
        // reading from tape should push the value onto the stack
        let mut process = Process::new_dummy_with_advice_tape(&[3]);
        process.execute_op(&Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(&Operation::Read).unwrap();
        let expected = build_expected_stack(&[3, 1]);
        assert_eq!(expected, process.stack.trace_state());

        // reading again should result in an error because advice tape is empty
        assert!(process.execute_op(&Operation::Read).is_err());
    }

    #[test]
    fn op_readw() {
        // reading from tape should overwrite top 4 values
        let mut process = Process::new_dummy_with_advice_tape(&[3, 4, 5, 6]);
        process.execute_op(&Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(&Operation::Pad).unwrap();
        process.execute_op(&Operation::Pad).unwrap();
        process.execute_op(&Operation::Pad).unwrap();
        process.execute_op(&Operation::Pad).unwrap();
        process.execute_op(&Operation::ReadW).unwrap();
        let expected = build_expected_stack(&[6, 5, 4, 3, 1]);
        assert_eq!(expected, process.stack.trace_state());

        // reading again should result in an error because advice tape is empty
        assert!(process.execute_op(&Operation::ReadW).is_err());

        // should return an error if the stack has fewer than 4 values
        let mut process = Process::new_dummy_with_advice_tape(&[3, 4, 5, 6]);
        process.execute_op(&Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(&Operation::Pad).unwrap();
        process.execute_op(&Operation::Pad).unwrap();
        assert!(process.execute_op(&Operation::ReadW).is_err());
    }

    // ENVIRONMENT INPUT TESTS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_sdepth() {
        // stack is empty
        let mut process = Process::new_dummy();
        process.execute_op(&Operation::SDepth).unwrap();
        let expected = build_expected_stack(&[0]);
        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(1, process.stack.depth());

        // stack has one item
        process.execute_op(&Operation::SDepth).unwrap();
        let expected = build_expected_stack(&[1, 0]);
        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(2, process.stack.depth());

        // stack has 3 items
        process.execute_op(&Operation::Pad).unwrap();
        process.execute_op(&Operation::SDepth).unwrap();
        let expected = build_expected_stack(&[3, 0, 1, 0]);
        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(4, process.stack.depth());
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn store_value(process: &mut Process, addr: u64, value: [Felt; 4]) {
        for &value in value.iter() {
            process.execute_op(&Operation::Push(value)).unwrap();
        }
        let addr = Felt::new(addr);
        process.execute_op(&Operation::Push(addr)).unwrap();
        process.execute_op(&Operation::StoreW).unwrap();
    }

    fn build_expected_stack(values: &[u64]) -> [Felt; 16] {
        let mut expected = [Felt::ZERO; 16];
        for (&value, result) in values.iter().zip(expected.iter_mut()) {
            *result = Felt::new(value);
        }
        expected
    }
}
