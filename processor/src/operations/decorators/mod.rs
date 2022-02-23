use super::{
    AdviceInjector, DebugOptions, ExecutionError, Felt, ProcInfo, Process, StarkField, Word,
};
use core::ops::RangeInclusive;
use log::info;

#[cfg(test)]
mod debug_tests;

// DECORATORS
// ================================================================================================

impl Process {
    // DEBUGGING
    // --------------------------------------------------------------------------------------------

    /// Inject procedure information into the procedure stack for tracking.
    pub fn op_proc_start(&mut self, info: &ProcInfo) -> Result<(), ExecutionError> {
        self.proc_stack.push(info.clone());
        Ok(())
    }

    /// Remove procedure information from the procedure stack on procedure exit.
    pub fn op_proc_end(&mut self) -> Result<(), ExecutionError> {
        self.proc_stack.pop().expect("no procedures in stack");
        Ok(())
    }

    /// Prints out debugging information based on options passed.
    pub fn op_debug(&mut self, options: &DebugOptions) -> Result<(), ExecutionError> {
        info!(
            "---------------------cycle: {}---------------------",
            self.system.clk()
        );
        match *options {
            DebugOptions::All => {
                self.print_stack(None);
                self.print_mem(None, None);
                self.print_local();
            }
            DebugOptions::Stack(n) => self.print_stack(n),
            DebugOptions::Memory(n, m) => self.print_mem(n, m),
            DebugOptions::Local(_) => self.print_local(),
        }

        Ok(())
    }

    // DEBUG HELPER
    // ---------------------------------------------------------------------------------

    /// Prints stack information for debugging.
    /// If n is passed, this prints the top n items in the stack.
    fn print_stack(&self, n: Option<usize>) {
        let states = self.stack.get_values(n);
        let values = states
            .iter()
            .map(|v| v.as_int().to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let depth = self.stack.depth();
        info!("stack ({} of {}) ---------", n.unwrap_or(depth), depth);
        info!("{}", values);
    }

    /// Create the display string for printing a word.
    /// Prints <empty> if no value exists.
    fn fmt_word(word: Option<Word>) -> String {
        let shorten = |v: Felt| format!("{:#016x}", v.as_int());
        match word {
            Some(v) => format!(
                "[{}, {}, {}, {}]",
                shorten(v[0]),
                shorten(v[1]),
                shorten(v[2]),
                shorten(v[3])
            ),
            None => String::from("<empty>"),
        }
    }

    /// Print the local variable that fmp pointer is referring to in memory.
    fn print_local(&self) {
        let local = self.memory.get_value(self.system.fmp().as_int());
        info!("local: {}", Process::fmt_word(local));
    }

    /// Print memory with optional starting and ending addresses.
    fn print_mem(&self, n: Option<u64>, m: Option<u64>) {
        // Convert vec of words into options
        let convert_vec = |v: Vec<(u64, Word)>| {
            v.into_iter()
                .map(|(k, v)| (k, Some(v)))
                .collect::<Vec<(u64, Option<Word>)>>()
        };
        let values: Vec<(u64, Option<Word>)> = match (n, m) {
            (Some(n), None) => {
                let value = self.memory.get_value(n);
                vec![(n, value)]
            }
            (Some(n), Some(m)) => convert_vec(self.memory.get_values(RangeInclusive::new(n, m))),
            (None, None) => convert_vec(self.memory.get_all_values()),
            _ => Vec::new(),
        };

        let size = values.iter().filter(|&n| n.1.is_some()).count();
        info!("memory ({} of {}) ---------", size, self.memory.size());

        for (address, value) in values {
            info!("{:#016x}: {}", address, Process::fmt_word(value));
        }
    }

    // ADVICE INJECTION
    // --------------------------------------------------------------------------------------------

    /// Process the specified advice injector.
    pub fn op_advice(&mut self, injector: AdviceInjector) -> Result<(), ExecutionError> {
        match injector {
            AdviceInjector::MerkleNode => self.inject_merkle_node(),
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
    /// - The stack contains fewer than 6 elements.
    /// - Merkle tree for the specified root cannot be found in the advice provider.
    /// - The specified depth is either zero or greater than the depth of the Merkle tree
    ///   identified by the specified root.
    /// - Value of the node at the specified depth and index is not known to the advice provider.
    fn inject_merkle_node(&mut self) -> Result<(), ExecutionError> {
        self.stack.check_depth(6, "INJMKNODE")?;

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

    use vm_core::{AdviceInjector, AdviceSet, ProgramInputs};

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
            .execute_op(&Operation::Advice(AdviceInjector::MerkleNode))
            .unwrap();
        // read the node from the tape onto the stack
        process.execute_op(&Operation::Read).unwrap();
        process.execute_op(&Operation::Read).unwrap();
        process.execute_op(&Operation::Read).unwrap();
        process.execute_op(&Operation::Read).unwrap();

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
