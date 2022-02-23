use super::{fmt, hasher, Digest, Felt, FieldElement, Operation};
use winter_utils::flatten_slice_elements;

// CONSTANTS
// ================================================================================================

/// Maximum number of operations per group.
const GROUP_SIZE: usize = 9;

/// Maximum number of groups per batch.
const BATCH_SIZE: usize = 8;

/// Maximum number of operations which can fit into a single operation batch.
const MAX_OPS_PER_BATCH: usize = GROUP_SIZE * BATCH_SIZE;

// SPAN BLOCK
// ================================================================================================
/// A code block used to describe a linear sequence of operations (i.e., no branching or loops).
///
/// When the VM executes a Span block, it breaks the sequence of operations into batches and
/// groups according to the following rules:
/// - A group may contain up to 9 operations or a single immediate value.
/// - A batch may contain up to 8 groups.
/// - There is no limit on the number of batches contained within a single span.
///
/// Thus, for example, executing 8 pushes in a row will result in two operation batches:
/// - The first batch will contain 8 groups, with the first group containing 7 push opcodes,
///   and the remaining 7 groups containing immediate values for each of the push operations.
/// - The second batch will contain 2 groups, with the first group containing a single push opcode,
///   and the second group containing the immediate value for the last push operation.
///
/// If a sequence of operations does not have any operations which carry immediate values, then
/// up to 72 operations can fit into a single batch.
///
/// From the user's perspective, all operations are executed in order, however, the assembler may
/// insert NOOPs to ensure proper alignment of all operations in the sequence.
///
/// TODO: describe how Span hash is computed.
#[derive(Clone, Debug)]
pub struct Span {
    op_batches: Vec<OpBatch>,
    hash: Digest,
}

impl Span {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [Span] block instantiated with the specified operations.
    ///
    /// # Errors (TODO)
    /// Returns an error if:
    /// - `operations` vector is empty.
    /// - `operations` vector contains any number of system operations.
    pub fn new(operations: Vec<Operation>) -> Self {
        assert!(!operations.is_empty()); // TODO: return error
        let (op_batches, hash) = batch_ops(operations);
        Self { op_batches, hash }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a hash of this code block.
    pub fn hash(&self) -> Digest {
        self.hash
    }

    /// Returns list of operation batches contained in this span block.
    pub fn op_batches(&self) -> &[OpBatch] {
        &self.op_batches
    }

    /// Returns a list of operations contained in this span block.
    pub fn get_ops(&self) -> Vec<Operation> {
        let mut ops = Vec::with_capacity(self.op_batches.len() * MAX_OPS_PER_BATCH);
        for batch in self.op_batches.iter() {
            ops.extend_from_slice(&batch.ops);
        }
        ops
    }

    // SPAN MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new [Span] block instantiated with operations from this block repeated the
    /// specified number of times.
    #[must_use]
    pub fn replicate(&self, num_copies: usize) -> Self {
        let own_ops = self.get_ops();
        let mut ops = Vec::with_capacity(own_ops.len() * num_copies);
        for _ in 0..num_copies {
            ops.extend_from_slice(&own_ops);
        }
        Self::new(ops)
    }

    /// Appends the operations from the provided [Span] to this [Span].
    pub fn append(&mut self, other: Self) {
        let mut ops = self.get_ops();
        for batch in other.op_batches {
            ops.extend_from_slice(&batch.ops);
        }
        let (op_batches, hash) = batch_ops(ops);
        self.op_batches = op_batches;
        self.hash = hash;
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "span")?;
        for batch in self.op_batches.iter() {
            for op in batch.ops.iter() {
                write!(f, " {}", op)?;
            }
        }
        write!(f, " end")
    }
}

// OPERATION BATCH
// ================================================================================================

/// A batch of operations in a [Span] block.
///
/// An operation batch consists of up to 8 operation groups, with each group containing up to 9
/// operations or a single immediate value.
#[derive(Clone, Debug)]
pub struct OpBatch {
    ops: Vec<Operation>,
    groups: [Felt; BATCH_SIZE],
}

impl OpBatch {
    /// Returns a new operation batch.
    fn new() -> Self {
        Self {
            ops: Vec::new(),
            groups: [Felt::ZERO; BATCH_SIZE],
        }
    }

    /// Returns a list of operations contained in this batch.
    pub fn ops(&self) -> &[Operation] {
        &self.ops
    }

    /// Returns a list of operation groups contained in this batch.
    ///
    /// Each group is represented by a single field element.
    pub fn groups(&self) -> [Felt; BATCH_SIZE] {
        self.groups
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn batch_ops(ops: Vec<Operation>) -> (Vec<OpBatch>, Digest) {
    // encodes a group of opcodes; up to 7 opcodes can fit into one group
    let mut op_group = 0u64;

    // index of the next opcode in the current group; valued range of values is [0, 8)
    let mut op_idx = 0;

    // indexes of the current group in the batch, and the next free group in the batch; these
    // could be different by more than 1 if some operations in the batch carry immediate values.
    let mut group_idx = 0;
    let mut next_group_idx = 1;

    // current batch of operations; each batch can contain a combination of op groups and immediate
    // values; total number of slots in a batch is 8.
    let mut batch = OpBatch::new();
    let mut batches = Vec::<OpBatch>::new();
    let mut batch_groups = Vec::<[Felt; BATCH_SIZE]>::new();

    for op in ops {
        // if the operator is a decorator we add it to the list of batch ops, but don't process it
        // further (i.e., the operation will not affect program hash).
        if op.is_decorator() {
            batch.ops.push(op);
            continue;
        }

        // if the operation carries immediate value, process it first
        if let Some(imm) = op.imm_value() {
            // check if the batch has room for the immediate value, and if not start another batch
            if next_group_idx >= BATCH_SIZE {
                batch_groups.push(batch.groups());
                batches.push(batch);
                batch = OpBatch::new();
                op_group = 0;
                group_idx = 0;
                next_group_idx = 1;
                op_idx = 0;
            }

            // operation carrying an immediate value cannot be the first one in the group except
            // for the first group of a batch; so, we just add a noop in front of it
            if op_idx == 0 && group_idx != 0 {
                batch.ops.push(Operation::Noop);
                op_group = Operation::Noop.op_code().expect("no opcode") as u64;
                op_idx += 1;
            }

            // put the immediate value into the next available slot
            batch.groups[next_group_idx] = imm;
            next_group_idx += 1;
        }

        // add the opcode to the group; the operation should have an opcode because we filter
        // out decorators at the beginning of the loop.
        op_group |= (op.op_code().expect("no opcode") as u64) << (Operation::OP_BITS * op_idx);
        batch.ops.push(op);
        op_idx += 1;

        // if the group is full, put it into the batch and start another group
        if op_idx == GROUP_SIZE {
            batch.groups[group_idx] = Felt::new(op_group);
            op_idx = 0;
            op_group = 0;
            group_idx = next_group_idx;
            next_group_idx += 1;

            // if the batch is full, start another batch
            if next_group_idx >= BATCH_SIZE {
                batch_groups.push(batch.groups());
                batches.push(batch);
                batch = OpBatch::new();
                group_idx = 0;
                next_group_idx = 1;
            }
        }
    }

    // make sure we finished processing the last batch
    if group_idx != 0 || op_idx != 0 {
        if op_group != 0 {
            batch.groups[group_idx] = Felt::new(op_group);
        }
        batch_groups.push(batch.groups());
        batches.push(batch);
    }

    let hash = hasher::hash_elements(flatten_slice_elements(&batch_groups));

    (batches, hash)
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{hasher, Felt, FieldElement, Operation, BATCH_SIZE};
    use crate::DebugOptions;

    #[test]
    fn batch_ops() {
        // one operation
        let ops = vec![Operation::Add];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());
        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        let mut batch_groups = [Felt::ZERO; BATCH_SIZE];
        batch_groups[0] = build_group(&ops);
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);

        // two operations
        let ops = vec![Operation::Add, Operation::Mul];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());
        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        let mut batch_groups = [Felt::ZERO; BATCH_SIZE];
        batch_groups[0] = build_group(&ops);
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);

        // one group with one immediate value
        let ops = vec![Operation::Add, Operation::Push(Felt::new(12345678))];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());
        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        let mut batch_groups = [Felt::ZERO; BATCH_SIZE];
        batch_groups[0] = build_group(&ops);
        batch_groups[1] = Felt::new(12345678);
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);

        // one group with 7 immediate values
        let ops = vec![
            Operation::Add,
            Operation::Push(Felt::new(1)),
            Operation::Push(Felt::new(2)),
            Operation::Push(Felt::new(3)),
            Operation::Push(Felt::new(4)),
            Operation::Push(Felt::new(5)),
            Operation::Push(Felt::new(6)),
            Operation::Push(Felt::new(7)),
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());
        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        let batch_groups = [
            build_group(&ops),
            Felt::new(1),
            Felt::new(2),
            Felt::new(3),
            Felt::new(4),
            Felt::new(5),
            Felt::new(6),
            Felt::new(7),
        ];
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);

        // two groups with 7 immediate values; the last push overflows to the second batch
        let ops = vec![
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Push(Felt::new(1)),
            Operation::Push(Felt::new(2)),
            Operation::Push(Felt::new(3)),
            Operation::Push(Felt::new(4)),
            Operation::Push(Felt::new(5)),
            Operation::Push(Felt::new(6)),
            Operation::Push(Felt::new(7)),
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(2, batches.len());
        let batch0 = &batches[0];
        assert_eq!(ops[..9], batch0.ops);

        let batch0_groups = [
            build_group(&ops[..9]),
            Felt::new(1),
            Felt::new(2),
            Felt::new(3),
            Felt::new(4),
            Felt::new(5),
            Felt::new(6),
            Felt::ZERO,
        ];
        assert_eq!(batch0_groups, batch0.groups);

        let batch1 = &batches[1];
        assert_eq!(vec![ops[9].clone()], batch1.ops);
        let mut batch1_groups = [Felt::ZERO; BATCH_SIZE];
        batch1_groups[0] = build_group(&[ops[9].clone()]);
        batch1_groups[1] = Felt::new(7);
        assert_eq!(batch1_groups, batch1.groups);

        let all_groups = [batch0_groups, batch1_groups].concat();
        assert_eq!(hasher::hash_elements(&all_groups), hash);

        // immediate values in-between groups
        let ops = vec![
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Push(Felt::new(7)),
            Operation::Add,
            Operation::Add,
            Operation::Push(Felt::new(11)),
            Operation::Mul,
            Operation::Mul,
            Operation::Add,
        ];

        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());
        let batch = &batches[0];
        assert_eq!(ops, batch.ops);

        let batch_groups = [
            build_group(&ops[..9]),
            Felt::new(7),
            Felt::new(11),
            build_group(&ops[9..]),
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
        ];
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);

        // push at start of second group; assembler inserts a NOOP in front of PUSH
        let ops = vec![
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Add,
            Operation::Add,
            Operation::Mul,
            Operation::Mul,
            Operation::Add,
            Operation::Add,
            Operation::Push(Felt::new(11)),
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());
        let batch = &batches[0];
        let mut expected_ops = ops.clone();
        expected_ops.insert(9, Operation::Noop);
        assert_eq!(expected_ops, batch.ops);
        let batch_groups = [
            build_group(&ops[..9]),
            build_group(&[Operation::Noop, ops[9].clone()]),
            Felt::new(11),
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
        ];
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);
    }

    #[test]
    fn batch_ops_with_decorator() {
        let ops = vec![
            Operation::Push(Felt::ONE),
            Operation::Add,
            Operation::Debug(DebugOptions::All),
            Operation::Mul,
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());
        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        let mut batch_groups = [Felt::ZERO; BATCH_SIZE];
        batch_groups[0] = build_group(&ops);
        batch_groups[1] = Felt::ONE;
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------

    fn build_group(ops: &[Operation]) -> Felt {
        let mut group = 0u64;
        let mut i = 0;
        for op in ops.iter() {
            if !op.is_decorator() {
                group |= (op.op_code().unwrap() as u64) << (Operation::OP_BITS * i);
                i += 1;
            }
        }
        Felt::new(group)
    }
}
