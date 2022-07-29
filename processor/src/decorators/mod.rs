use super::{AdviceInjector, Decorator, ExecutionError, Felt, Process, StarkField};

// DECORATORS
// ================================================================================================

impl Process {
    /// Executes the specified decorator
    pub(super) fn execute_decorator(
        &mut self,
        decorator: &Decorator,
    ) -> Result<(), ExecutionError> {
        match decorator {
            Decorator::Advice(injector) => self.dec_advice(injector)?,
            Decorator::AsmOp(assembly_op) => {
                if self.decoder.in_debug_mode() {
                    self.decoder
                        .append_asmop(self.system.clk(), assembly_op.clone());
                }
            }
            Decorator::ProcMarker(_) => {
                if self.decoder.in_debug_mode() {
                    self.decoder
                        .append_proc_marker(self.system.clk(), decorator.clone());
                }
            }
        }
        Ok(())
    }

    // ADVICE INJECTION
    // --------------------------------------------------------------------------------------------

    /// Process the specified advice injector.
    pub fn dec_advice(&mut self, injector: &AdviceInjector) -> Result<(), ExecutionError> {
        match injector {
            AdviceInjector::MerkleNode => self.inject_merkle_node(),
            AdviceInjector::DivResultU64 => self.inject_div_result_u64(),
        }
    }

    // INJECTOR HELPERS
    // --------------------------------------------------------------------------------------------

    /// Injects a node of the Merkle tree specified by the values on the stack at the head of the
    /// advice tape. The stack is expected to be arranged as follows (from the top):
    /// - depth of the node, 1 element
    /// - index of the node, 1 element
    /// - root of the tree, 4 elements
    ///
    /// # Errors
    /// Returns an error if:
    /// - Merkle tree for the specified root cannot be found in the advice provider.
    /// - The specified depth is either zero or greater than the depth of the Merkle tree
    ///   identified by the specified root.
    /// - Value of the node at the specified depth and index is not known to the advice provider.
    fn inject_merkle_node(&mut self) -> Result<(), ExecutionError> {
        // read node depth, node index, and tree root from the stack
        let depth = self.stack.get(0);
        let index = self.stack.get(1);
        let root = [
            self.stack.get(5),
            self.stack.get(4),
            self.stack.get(3),
            self.stack.get(2),
        ];

        // look up the node in the advice provider
        let node = self.advice.get_tree_node(root, depth, index)?;

        // write the node into the advice tape with first element written last so that it can be
        // removed first
        self.advice.write_tape(node[3]);
        self.advice.write_tape(node[2]);
        self.advice.write_tape(node[1]);
        self.advice.write_tape(node[0]);

        Ok(())
    }

    /// Injects the result of u64 division (both the quotient and the remainder) at the head of
    /// the advice tape. The stack is expected to be arranged as follows (from the top):
    /// - divisor split into two 32-bit elements
    /// - dividend split into two 32-bit elements
    ///
    /// The result is injected into the advice tape as follows: first the remainder is injected,
    /// then the quotient is injected. This guarantees that when reading values from the advice
    /// tape, first the quotient will be read, and then the remainder.
    ///
    /// # Errors
    /// Returns an error if the divisor is ZERO.
    fn inject_div_result_u64(&mut self) -> Result<(), ExecutionError> {
        let divisor_hi = self.stack.get(0).as_int();
        let divisor_lo = self.stack.get(1).as_int();
        let divisor = (divisor_hi << 32) + divisor_lo;

        if divisor == 0 {
            return Err(ExecutionError::DivideByZero(self.system.clk()));
        }

        let dividend_hi = self.stack.get(2).as_int();
        let dividend_lo = self.stack.get(3).as_int();
        let dividend = (dividend_hi << 32) + dividend_lo;

        let quotient = dividend / divisor;
        let remainder = dividend - quotient * divisor;

        let (q_hi, q_lo) = u64_to_u32_elements(quotient);
        let (r_hi, r_lo) = u64_to_u32_elements(remainder);

        self.advice.write_tape(r_hi);
        self.advice.write_tape(r_lo);
        self.advice.write_tape(q_hi);
        self.advice.write_tape(q_lo);

        Ok(())
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn u64_to_u32_elements(value: u64) -> (Felt, Felt) {
    let hi = Felt::new(value >> 32);
    let lo = Felt::new((value as u32) as u64);
    (hi, lo)
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{
        super::{Felt, FieldElement, Operation, StarkField},
        Process,
    };
    use crate::Word;

    use vm_core::{AdviceInjector, AdviceSet, Decorator, ProgramInputs};

    #[test]
    fn inject_merkle_node() {
        let leaves = [init_leaf(1), init_leaf(2), init_leaf(3), init_leaf(4)];

        let tree = AdviceSet::new_merkle_tree(leaves.to_vec()).unwrap();
        let stack_inputs = [
            tree.root()[0].as_int(),
            tree.root()[1].as_int(),
            tree.root()[2].as_int(),
            tree.root()[3].as_int(),
            1,
            tree.depth() as u64,
        ];

        let inputs = ProgramInputs::new(&stack_inputs, &[], vec![tree.clone()]).unwrap();
        let mut process = Process::new(inputs);

        // inject the node into the advice tape
        process
            .execute_decorator(&Decorator::Advice(AdviceInjector::MerkleNode))
            .unwrap();

        // read the node from the tape onto the stack
        process.execute_op(Operation::Read).unwrap();
        process.execute_op(Operation::Read).unwrap();
        process.execute_op(Operation::Read).unwrap();
        process.execute_op(Operation::Read).unwrap();

        let expected_stack = build_expected(&[
            leaves[1][3],
            leaves[1][2],
            leaves[1][1],
            leaves[1][0],
            Felt::new(2),
            Felt::new(1),
            tree.root()[3],
            tree.root()[2],
            tree.root()[1],
            tree.root()[0],
        ]);
        assert_eq!(expected_stack, process.stack.trace_state());
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------
    fn init_leaf(value: u64) -> Word {
        [Felt::new(value), Felt::ZERO, Felt::ZERO, Felt::ZERO]
    }

    fn build_expected(values: &[Felt]) -> [Felt; 16] {
        let mut expected = [Felt::ZERO; 16];
        for (&value, result) in values.iter().zip(expected.iter_mut()) {
            *result = value
        }
        expected
    }
}
