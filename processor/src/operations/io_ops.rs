use super::{ExecutionError, Felt, Operation, Process};

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
    /// - The top four elements of the stack are overwritten with values retrieved from memory.
    ///
    /// Thus, the net result of the operation is that the stack is shifted left by one item.
    pub(super) fn op_mloadw(&mut self) -> Result<(), ExecutionError> {
        // get the address from the stack and read the word from memory
        let addr = self.stack.get(0);
        let word = self.chiplets.read_mem(addr);

        // update the stack state
        for (i, &value) in word.iter().rev().enumerate() {
            self.stack.set(i, value);
        }
        self.stack.shift_left(5);

        Ok(())
    }

    /// Loads the first element from the specified memory address onto the stack.
    ///
    /// The operation works as follows:
    /// - The memory address is popped off the stack.
    /// - A word is retrieved from memory at the specified address. The memory is always
    ///   initialized to ZEROs, and thus, if the specified address has never been written to,
    ///   four ZERO elements are returned.
    /// - The first element of the word retrieved from memory is pushed to the top of the stack.
    ///
    /// The first 3 helper registers are filled with the elements of the word which were not pushed
    /// to the stack.
    pub(super) fn op_mload(&mut self) -> Result<(), ExecutionError> {
        // get the address from the stack and read the word from memory
        let addr = self.stack.get(0);
        let word = self.chiplets.read_mem(addr);

        // update the stack state
        self.stack.set(0, word[0]);
        self.stack.copy_state(1);

        self.decoder
            .set_user_op_helpers(Operation::MLoad, &word[1..]);

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
    /// The first 4 helper registers are filled with the values which were stored in memory before
    /// the operation.
    pub(super) fn op_mstorew(&mut self) -> Result<(), ExecutionError> {
        // get the address from the stack and build the word to be saved from the stack values
        let addr = self.stack.get(0);

        let word = [
            self.stack.get(4),
            self.stack.get(3),
            self.stack.get(2),
            self.stack.get(1),
        ];

        // write the word to memory and get the previous word
        let old_word = self.chiplets.write_mem(addr, word);

        // update the stack state
        for (i, &value) in word.iter().rev().enumerate() {
            self.stack.set(i, value);
        }
        self.stack.shift_left(5);

        self.decoder
            .set_user_op_helpers(Operation::MStoreW, &old_word);

        Ok(())
    }

    /// Stores an element from the stack into the first slot at the specified memory address.
    ///
    /// The operation works as follows:
    /// - The memory address is popped off the stack.
    /// - The top stack element is saved into the first element of the word located at the specified
    /// memory address. The remaining 3 elements of the word are not affected. The element is not
    /// removed from the stack.
    ///
    /// Thus, the net result of the operation is that the stack is shifted left by one item.
    ///
    /// The first 4 helper registers are filled with the values which were stored in memory before
    /// the operation.
    pub(super) fn op_mstore(&mut self) -> Result<(), ExecutionError> {
        // get the address and the value from the stack
        let addr = self.stack.get(0);
        let value = self.stack.get(1);

        // write the value to the memory and get the previous word
        let old_word = self.chiplets.write_mem_single(addr, value);

        self.decoder
            .set_user_op_helpers(Operation::MStore, &old_word);

        // update the stack state
        self.stack.shift_left(1);

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
    /// Returns an error if the advice tape contains fewer than four elements.
    pub(super) fn op_readw(&mut self) -> Result<(), ExecutionError> {
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
    use vm_core::{utils::ToElements, MIN_STACK_DEPTH, ZERO};

    #[test]
    fn op_push() {
        let mut process = Process::new_dummy();
        assert_eq!(MIN_STACK_DEPTH, process.stack.depth());
        assert_eq!(0, process.stack.current_clk());
        assert_eq!([ZERO; 16], process.stack.trace_state());

        // push one item onto the stack
        let op = Operation::Push(Felt::ONE);
        process.execute_op(op).unwrap();
        let mut expected = [ZERO; 16];
        expected[0] = Felt::ONE;

        assert_eq!(MIN_STACK_DEPTH + 1, process.stack.depth());
        assert_eq!(1, process.stack.current_clk());
        assert_eq!(expected, process.stack.trace_state());

        // push another item onto the stack
        let op = Operation::Push(Felt::new(3));
        process.execute_op(op).unwrap();
        let mut expected = [ZERO; 16];
        expected[0] = Felt::new(3);
        expected[1] = Felt::ONE;

        assert_eq!(MIN_STACK_DEPTH + 2, process.stack.depth());
        assert_eq!(2, process.stack.current_clk());
        assert_eq!(expected, process.stack.trace_state());
    }

    // MEMORY OPERATION TESTS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_mstorew() {
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert_eq!(0, process.chiplets.get_mem_size());

        // push the first word onto the stack and save it at address 0
        let word1 = [1, 3, 5, 7].to_elements().try_into().unwrap();
        store_value(&mut process, 0, word1);

        // check stack state
        let expected_stack = build_expected_stack(&[7, 5, 3, 1]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        assert_eq!(1, process.chiplets.get_mem_size());
        assert_eq!(word1, process.chiplets.get_mem_value(0).unwrap());

        // push the second word onto the stack and save it at address 3
        let word2 = [2, 4, 6, 8].to_elements().try_into().unwrap();
        store_value(&mut process, 3, word2);

        // check stack state
        let expected_stack = build_expected_stack(&[8, 6, 4, 2, 7, 5, 3, 1]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        assert_eq!(2, process.chiplets.get_mem_size());
        assert_eq!(word1, process.chiplets.get_mem_value(0).unwrap());
        assert_eq!(word2, process.chiplets.get_mem_value(3).unwrap());

        // --- calling STOREW with a stack of minimum depth is ok ----------------
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert!(process.execute_op(Operation::MStoreW).is_ok());
    }

    #[test]
    fn op_mloadw() {
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert_eq!(0, process.chiplets.get_mem_size());

        // push a word onto the stack and save it at address 1
        let word = [1, 3, 5, 7].to_elements().try_into().unwrap();
        store_value(&mut process, 1, word);

        // push four zeros onto the stack
        for _ in 0..4 {
            process.execute_op(Operation::Pad).unwrap();
        }

        // push the address onto the stack and load the word
        process.execute_op(Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(Operation::MLoadW).unwrap();

        let expected_stack = build_expected_stack(&[7, 5, 3, 1, 7, 5, 3, 1]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        assert_eq!(1, process.chiplets.get_mem_size());
        assert_eq!(word, process.chiplets.get_mem_value(1).unwrap());

        // --- calling LOADW with a stack of minimum depth is ok ----------------
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert!(process.execute_op(Operation::MLoadW).is_ok());
    }

    #[test]
    fn op_mload() {
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert_eq!(0, process.chiplets.get_mem_size());

        // push a word onto the stack and save it at address 2
        let word = [1, 3, 5, 7].to_elements().try_into().unwrap();
        store_value(&mut process, 2, word);

        // push the address onto the stack and load the element
        process.execute_op(Operation::Push(Felt::new(2))).unwrap();
        process.execute_op(Operation::MLoad).unwrap();

        let expected_stack = build_expected_stack(&[1, 7, 5, 3, 1]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        assert_eq!(1, process.chiplets.get_mem_size());
        assert_eq!(word, process.chiplets.get_mem_value(2).unwrap());

        // --- calling MLOAD with a stack of minimum depth is ok ----------------
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert!(process.execute_op(Operation::MLoad).is_ok());
    }

    #[test]
    fn op_mstore() {
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert_eq!(0, process.chiplets.get_mem_size());

        // push new element onto the stack and save it as first element of the word on
        // uninitialized memory at address 0
        let element = Felt::new(10);
        store_element(&mut process, 0, element);

        // check stack state
        let expected_stack = build_expected_stack(&[10]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state
        let mem_0 = [element, ZERO, ZERO, ZERO];
        assert_eq!(1, process.chiplets.get_mem_size());
        assert_eq!(mem_0, process.chiplets.get_mem_value(0).unwrap());

        // push the word onto the stack and save it at address 2
        let word_2 = [1, 3, 5, 7].to_elements().try_into().unwrap();
        store_value(&mut process, 2, word_2);

        // push new element onto the stack and save it as first element of the word at address 2
        let element = Felt::new(12);
        store_element(&mut process, 2, element);

        // check stack state
        let expected_stack = build_expected_stack(&[12, 7, 5, 3, 1, 10]);
        assert_eq!(expected_stack, process.stack.trace_state());

        // check memory state to make sure the other 3 elements were not affected
        let mem_2 = [element, Felt::new(3), Felt::new(5), Felt::new(7)];
        assert_eq!(2, process.chiplets.get_mem_size());
        assert_eq!(mem_2, process.chiplets.get_mem_value(2).unwrap());

        // --- calling MSTORE with a stack of minimum depth is ok ----------------
        let mut process = Process::new_dummy_with_decoder_helpers();
        assert!(process.execute_op(Operation::MStore).is_ok());
    }

    // ADVICE INPUT TESTS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_read() {
        // reading from tape should push the value onto the stack
        let mut process = Process::new_dummy_with_advice_tape(&[3]);
        process.execute_op(Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(Operation::Read).unwrap();
        let expected = build_expected_stack(&[3, 1]);
        assert_eq!(expected, process.stack.trace_state());

        // reading again should result in an error because advice tape is empty
        assert!(process.execute_op(Operation::Read).is_err());
    }

    #[test]
    fn op_readw() {
        // reading from tape should overwrite top 4 values
        let mut process = Process::new_dummy_with_advice_tape(&[3, 4, 5, 6]);
        process.execute_op(Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(Operation::Pad).unwrap();
        process.execute_op(Operation::Pad).unwrap();
        process.execute_op(Operation::Pad).unwrap();
        process.execute_op(Operation::Pad).unwrap();
        process.execute_op(Operation::ReadW).unwrap();
        let expected = build_expected_stack(&[6, 5, 4, 3, 1]);
        assert_eq!(expected, process.stack.trace_state());

        // reading again should result in an error because advice tape is empty
        assert!(process.execute_op(Operation::ReadW).is_err());

        // should not return an error if the stack has fewer than 4 values
        let mut process = Process::new_dummy_with_advice_tape(&[3, 4, 5, 6]);
        process.execute_op(Operation::Push(Felt::ONE)).unwrap();
        process.execute_op(Operation::Pad).unwrap();
        process.execute_op(Operation::Pad).unwrap();
        assert!(process.execute_op(Operation::ReadW).is_ok());
        let expected = build_expected_stack(&[6, 5, 4, 3]);
        assert_eq!(expected, process.stack.trace_state());
    }

    // ENVIRONMENT INPUT TESTS
    // --------------------------------------------------------------------------------------------

    #[test]
    fn op_sdepth() {
        // stack is empty
        let mut process = Process::new_dummy();
        process.execute_op(Operation::SDepth).unwrap();
        let expected = build_expected_stack(&[MIN_STACK_DEPTH as u64]);
        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(MIN_STACK_DEPTH + 1, process.stack.depth());

        // stack has one item
        process.execute_op(Operation::SDepth).unwrap();
        let expected = build_expected_stack(&[MIN_STACK_DEPTH as u64 + 1, MIN_STACK_DEPTH as u64]);
        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(MIN_STACK_DEPTH + 2, process.stack.depth());

        // stack has 3 items
        process.execute_op(Operation::Pad).unwrap();
        process.execute_op(Operation::SDepth).unwrap();
        let expected = build_expected_stack(&[
            MIN_STACK_DEPTH as u64 + 3,
            0,
            MIN_STACK_DEPTH as u64 + 1,
            MIN_STACK_DEPTH as u64,
        ]);
        assert_eq!(expected, process.stack.trace_state());
        assert_eq!(MIN_STACK_DEPTH + 4, process.stack.depth());
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn store_value(process: &mut Process, addr: u64, value: [Felt; 4]) {
        for &value in value.iter() {
            process.execute_op(Operation::Push(value)).unwrap();
        }
        let addr = Felt::new(addr);
        process.execute_op(Operation::Push(addr)).unwrap();
        process.execute_op(Operation::MStoreW).unwrap();
    }

    fn store_element(process: &mut Process, addr: u64, value: Felt) {
        process.execute_op(Operation::Push(value)).unwrap();
        let addr = Felt::new(addr);
        process.execute_op(Operation::Push(addr)).unwrap();
        process.execute_op(Operation::MStore).unwrap();
    }

    fn build_expected_stack(values: &[u64]) -> [Felt; 16] {
        let mut expected = [ZERO; 16];
        for (&value, result) in values.iter().zip(expected.iter_mut()) {
            *result = Felt::new(value);
        }
        expected
    }
}
