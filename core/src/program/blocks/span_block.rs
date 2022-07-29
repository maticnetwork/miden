use super::{fmt, hasher, Digest, Felt, FieldElement, Operation, Vec};
use crate::{DecoratorIterator, DecoratorList};
use winter_utils::flatten_slice_elements;

// CONSTANTS
// ================================================================================================

/// Maximum number of operations per group.
pub const GROUP_SIZE: usize = 9;

/// Maximum number of groups per batch.
pub const BATCH_SIZE: usize = 8;

/// Maximum number of operations which can fit into a single operation batch.
const MAX_OPS_PER_BATCH: usize = GROUP_SIZE * BATCH_SIZE;

// SPAN BLOCK
// ================================================================================================
/// A code block used to describe a linear sequence of operations (i.e., no branching or loops).
///
/// When the VM executes a Span block, it breaks the sequence of operations into batches and
/// groups according to the following rules:
/// - A group may contain up to 9 operations or a single immediate value.
/// - An operation carrying an immediate value cannot be in the 9th position in a group.
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
/// TODO: describe how Span hash is computed.
#[derive(Clone, Debug)]
pub struct Span {
    op_batches: Vec<OpBatch>,
    hash: Digest,
    decorators: DecoratorList,
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
        Self::with_decorators(operations, DecoratorList::new())
    }

    /// Returns a new [Span] block instantiated with the specified operations and decorators.
    ///
    /// # Errors (TODO)
    /// Returns an error if:
    /// - `operations` vector is empty.
    /// - `operations` vector contains any number of system operations.
    pub fn with_decorators(operations: Vec<Operation>, decorators: DecoratorList) -> Self {
        assert!(!operations.is_empty()); // TODO: return error

        // validate decorators list (only in debug mode)
        #[cfg(debug_assertions)]
        validate_decorators(&operations, &decorators);

        let (op_batches, hash) = batch_ops(operations);
        Self {
            op_batches,
            hash,
            decorators,
        }
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

    // SPAN MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new [Span] block instantiated with operations from this block repeated the
    /// specified number of times.
    #[must_use]
    pub fn replicate(&self, num_copies: usize) -> Self {
        let own_ops = self.get_ops();
        let own_decorators = &self.decorators;
        let mut ops = Vec::with_capacity(own_ops.len() * num_copies);
        let mut decorators = DecoratorList::new();

        for i in 0..num_copies {
            // replicate decorators of a span block
            for decorator in own_decorators {
                decorators.push((own_ops.len() * i + decorator.0, decorator.1.clone()))
            }
            ops.extend_from_slice(&own_ops);
        }
        Self::with_decorators(ops, decorators)
    }

    /// Returns a list of decorators in this span block
    pub fn decorators(&self) -> &DecoratorList {
        &self.decorators
    }

    /// Returns a [DecoratorIterator] which allows us to iterate through the decorator list of this span
    /// block while executing operation batches of this span block
    pub fn decorator_iter(&self) -> DecoratorIterator {
        DecoratorIterator::new(&self.decorators)
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns a list of operations contained in this span block.
    fn get_ops(&self) -> Vec<Operation> {
        let mut ops = Vec::with_capacity(self.op_batches.len() * MAX_OPS_PER_BATCH);
        for batch in self.op_batches.iter() {
            ops.extend_from_slice(&batch.ops);
        }
        ops
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
    op_counts: [usize; BATCH_SIZE],
    num_groups: usize,
}

impl OpBatch {
    /// Returns a list of operations contained in this batch.
    pub fn ops(&self) -> &[Operation] {
        &self.ops
    }

    /// Returns a list of operation groups contained in this batch.
    ///
    /// Each group is represented by a single field element.
    pub fn groups(&self) -> &[Felt; BATCH_SIZE] {
        &self.groups
    }

    /// Returns the number of non-decorator operations for each operation group.
    ///
    /// Number of operations for groups containing immediate values is set to 0.
    pub fn op_counts(&self) -> &[usize; BATCH_SIZE] {
        &self.op_counts
    }

    /// Returns the number of groups in this batch.
    pub fn num_groups(&self) -> usize {
        self.num_groups
    }
}

/// An accumulator used in construction of operation batches.
struct OpBatchAccumulator {
    /// A list of operations in this batch, including decorators.
    ops: Vec<Operation>,
    /// Values of operation groups, including immediate values.
    groups: [Felt; BATCH_SIZE],
    /// Number of non-decorator operations in each operation group. Operation count for groups
    /// with immediate values is set to 0.
    op_counts: [usize; BATCH_SIZE],
    /// Value of the currently active op group.
    group: u64,
    /// Index of the next opcode in the current group.
    op_idx: usize,
    /// index of the current group in the batch.
    group_idx: usize,
    // Index of the next free group in the batch.
    next_group_idx: usize,
}

impl OpBatchAccumulator {
    /// Returns a blank [OpBatchAccumulator].
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            groups: [Felt::ZERO; BATCH_SIZE],
            op_counts: [0; BATCH_SIZE],
            group: 0,
            op_idx: 0,
            group_idx: 0,
            next_group_idx: 1,
        }
    }

    /// Returns true if this accumulator does not contain any operations.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Returns true if this accumulator can accept the specified operation.
    ///
    /// An accumulator may not be able accept an operation for the following reasons:
    /// - There is no more space in the underlying batch (e.g., the 8th group of the batch
    ///   already contains 9 operations).
    /// - There is no space for the immediate value carried by the operation (e.g., the 8th
    ///   group is only partially full, but we are trying to add a PUSH operation).
    /// - The alignment rules require that the operation overflows into the next group, and
    ///   if this happens, there will be no space for the operation or its immediate value.
    pub fn can_accept_op(&self, op: Operation) -> bool {
        if op.imm_value().is_some() {
            // an operation carrying an immediate value cannot be the last one in a group; so, we
            // check if we need to move the operation to the next group. in either case, we need
            // to make sure there is enough space for the immediate value as well.
            if self.op_idx < GROUP_SIZE - 1 {
                self.next_group_idx < BATCH_SIZE
            } else {
                self.next_group_idx + 1 < BATCH_SIZE
            }
        } else {
            // check if there is space for the operation in the current group, or if there isn't,
            // whether we can add another group
            self.op_idx < GROUP_SIZE || self.next_group_idx < BATCH_SIZE
        }
    }

    /// Adds the specified operation to this accumulator. It is expected that the specified
    /// operation is not a decorator and that (can_accept_op())[OpBatchAccumulator::can_accept_op]
    /// is called before this function to make sure that the specified operation can be added to
    /// the accumulator.
    pub fn add_op(&mut self, op: Operation) {
        // if the group is full, finalize it and start a new group
        if self.op_idx == GROUP_SIZE {
            self.finalize_op_group();
        }

        // for operations with immediate values, we need to do a few more things
        if let Some(imm) = op.imm_value() {
            // since an operation with an immediate value cannot be the last one in a group, if
            // the operation would be the last one in the group, we need to start a new group
            if self.op_idx == GROUP_SIZE - 1 {
                self.finalize_op_group();
            }

            // save the immediate value at the next group index and advance the next group pointer
            self.groups[self.next_group_idx] = imm;
            self.next_group_idx += 1;
        }

        // add the opcode to the group and increment the op index pointer
        let opcode = op.op_code() as u64;
        self.group |= opcode << (Operation::OP_BITS * self.op_idx);
        self.ops.push(op);
        self.op_idx += 1;
    }

    /// Convert the accumulator into an [OpBatch].
    pub fn into_batch(mut self) -> OpBatch {
        // make sure the last group gets added to the group array; we also check the op_idx to
        // handle the case when a group contains a single NOOP operation.
        if self.group != 0 || self.op_idx != 0 {
            self.groups[self.group_idx] = Felt::new(self.group);
            self.op_counts[self.group_idx] = self.op_idx;
        }

        OpBatch {
            ops: self.ops,
            groups: self.groups,
            op_counts: self.op_counts,
            num_groups: self.next_group_idx,
        }
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    /// Saves the current group into the group array, advances current and next group pointers,
    /// and resets group content.
    fn finalize_op_group(&mut self) {
        self.groups[self.group_idx] = Felt::new(self.group);
        self.op_counts[self.group_idx] = self.op_idx;

        self.group_idx = self.next_group_idx;
        self.next_group_idx = self.group_idx + 1;

        self.op_idx = 0;
        self.group = 0;
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Groups the provided operations into batches as described in the docs for this module (i.e.,
/// up to 9 operations per group, and up to 8 groups per batch).
///
/// After the operations have been grouped, computes the hash of the block.
fn batch_ops(ops: Vec<Operation>) -> (Vec<OpBatch>, Digest) {
    let mut batch_acc = OpBatchAccumulator::new();
    let mut batches = Vec::<OpBatch>::new();
    let mut batch_groups = Vec::<[Felt; BATCH_SIZE]>::new();

    for op in ops {
        // if the operation cannot be accepted into the current accumulator, add the contents of
        // the accumulator to the list of batches and start a new accumulator
        if !batch_acc.can_accept_op(op) {
            let batch = batch_acc.into_batch();
            batch_acc = OpBatchAccumulator::new();

            batch_groups.push(*batch.groups());
            batches.push(batch);
        }

        // add the operation to the accumulator
        batch_acc.add_op(op);
    }

    // make sure we finished processing the last batch
    if !batch_acc.is_empty() {
        let batch = batch_acc.into_batch();
        batch_groups.push(*batch.groups());
        batches.push(batch);
    }

    // compute the hash of all operation groups
    let num_op_groups = get_span_op_group_count(&batches);
    let op_groups = &flatten_slice_elements(&batch_groups)[..num_op_groups];
    let hash = hasher::hash_elements(op_groups);

    (batches, hash)
}

/// Returns the total number of operation groups in a span defined by the provides list of
/// operation batches.
///
/// Then number of operation groups is computed as follows:
/// - For all batches but the last one we set the number of groups to 8, regardless of the
///   actual number of groups in the batch. The reason for this is that when operation
///   batches are concatenated together each batch contributes 8 elements to the hash.
/// - For the last batch, we take the number of actual batches and round it up to the next
///   power of two. The reason for rounding is that the VM always executes a number of
///   operation groups which is a power of two.
pub fn get_span_op_group_count(op_batches: &[OpBatch]) -> usize {
    let last_batch_num_groups = op_batches.last().expect("no last group").num_groups();
    (op_batches.len() - 1) * BATCH_SIZE + last_batch_num_groups.next_power_of_two()
}

/// Checks if a given decorators list is valid (only checked in debug mode)
/// - Assert the decorator list is in ascending order.
/// - Assert the last op index in decorator list is less than the number of operations.
#[cfg(debug_assertions)]
fn validate_decorators(operations: &Vec<Operation>, decorators: &DecoratorList) {
    if !decorators.is_empty() {
        // check if decorator list is sorted
        for i in 0..(decorators.len() - 1) {
            debug_assert!(
                decorators[i + 1].0 >= decorators[i].0,
                "unsorted decorators list"
            );
        }
        // assert the last index in decorator list is less than operations vector length
        debug_assert!(
            operations.len() > decorators.last().expect("empty decorators list").0,
            "last op index in decorator list should be less than number of ops"
        );
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{hasher, Felt, FieldElement, Operation, BATCH_SIZE};

    #[test]
    fn batch_ops() {
        // --- one operation ----------------------------------------------------------------------
        let ops = vec![Operation::Add];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());

        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        assert_eq!(1, batch.num_groups());

        let mut batch_groups = [Felt::ZERO; BATCH_SIZE];
        batch_groups[0] = build_group(&ops);

        assert_eq!(batch_groups, batch.groups);
        assert_eq!([1_usize, 0, 0, 0, 0, 0, 0, 0], batch.op_counts);
        assert_eq!(hasher::hash_elements(&batch_groups[..1]), hash);

        // --- two operations ---------------------------------------------------------------------
        let ops = vec![Operation::Add, Operation::Mul];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());

        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        assert_eq!(1, batch.num_groups());

        let mut batch_groups = [Felt::ZERO; BATCH_SIZE];
        batch_groups[0] = build_group(&ops);

        assert_eq!(batch_groups, batch.groups);
        assert_eq!([2_usize, 0, 0, 0, 0, 0, 0, 0], batch.op_counts);
        assert_eq!(hasher::hash_elements(&batch_groups[..1]), hash);

        // --- one group with one immediate value -------------------------------------------------
        let ops = vec![Operation::Add, Operation::Push(Felt::new(12345678))];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());

        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        assert_eq!(2, batch.num_groups());

        let mut batch_groups = [Felt::ZERO; BATCH_SIZE];
        batch_groups[0] = build_group(&ops);
        batch_groups[1] = Felt::new(12345678);

        assert_eq!(batch_groups, batch.groups);
        assert_eq!([2_usize, 0, 0, 0, 0, 0, 0, 0], batch.op_counts);
        assert_eq!(hasher::hash_elements(&batch_groups[..2]), hash);

        // --- one group with 7 immediate values --------------------------------------------------
        let ops = vec![
            Operation::Push(Felt::new(1)),
            Operation::Push(Felt::new(2)),
            Operation::Push(Felt::new(3)),
            Operation::Push(Felt::new(4)),
            Operation::Push(Felt::new(5)),
            Operation::Push(Felt::new(6)),
            Operation::Push(Felt::new(7)),
            Operation::Add,
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());

        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        assert_eq!(8, batch.num_groups());

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
        assert_eq!([8_usize, 0, 0, 0, 0, 0, 0, 0], batch.op_counts);
        assert_eq!(hasher::hash_elements(&batch_groups), hash);

        // --- two groups with 7 immediate values; the last push overflows to the second batch ----
        let ops = vec![
            Operation::Add,
            Operation::Mul,
            Operation::Push(Felt::new(1)),
            Operation::Push(Felt::new(2)),
            Operation::Push(Felt::new(3)),
            Operation::Push(Felt::new(4)),
            Operation::Push(Felt::new(5)),
            Operation::Push(Felt::new(6)),
            Operation::Add,
            Operation::Push(Felt::new(7)),
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(2, batches.len());

        let batch0 = &batches[0];
        assert_eq!(ops[..9], batch0.ops);
        assert_eq!(7, batch0.num_groups());

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
        assert_eq!([9_usize, 0, 0, 0, 0, 0, 0, 0], batch0.op_counts);

        let batch1 = &batches[1];
        assert_eq!(vec![ops[9]], batch1.ops);
        assert_eq!(2, batch1.num_groups());

        let mut batch1_groups = [Felt::ZERO; BATCH_SIZE];
        batch1_groups[0] = build_group(&[ops[9]]);
        batch1_groups[1] = Felt::new(7);

        assert_eq!([1_usize, 0, 0, 0, 0, 0, 0, 0], batch1.op_counts);
        assert_eq!(batch1_groups, batch1.groups);

        let all_groups = [batch0_groups, batch1_groups].concat();
        assert_eq!(hasher::hash_elements(&all_groups[..10]), hash);

        // --- immediate values in-between groups -------------------------------------------------
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
        assert_eq!(4, batch.num_groups());

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

        assert_eq!([9_usize, 0, 0, 1, 0, 0, 0, 0], batch.op_counts);
        assert_eq!(batch_groups, batch.groups);
        assert_eq!(hasher::hash_elements(&batch_groups[..4]), hash);

        // --- push at the end of a group is moved into the next group ----------------------------
        let ops = vec![
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Add,
            Operation::Add,
            Operation::Mul,
            Operation::Mul,
            Operation::Add,
            Operation::Push(Felt::new(11)),
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());

        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        assert_eq!(3, batch.num_groups());

        let batch_groups = [
            build_group(&ops[..8]),
            build_group(&[ops[8]]),
            Felt::new(11),
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
        ];

        assert_eq!(batch_groups, batch.groups);
        assert_eq!([8_usize, 1, 0, 0, 0, 0, 0, 0], batch.op_counts);
        assert_eq!(hasher::hash_elements(&batch_groups[..4]), hash);

        // --- push at the end of a group is moved into the next group ----------------------------
        let ops = vec![
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Add,
            Operation::Add,
            Operation::Mul,
            Operation::Mul,
            Operation::Push(Felt::new(1)),
            Operation::Push(Felt::new(2)),
        ];
        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(1, batches.len());

        let batch = &batches[0];
        assert_eq!(ops, batch.ops);
        assert_eq!(4, batch.num_groups());

        let batch_groups = [
            build_group(&ops[..8]),
            Felt::new(1),
            build_group(&[ops[8]]),
            Felt::new(2),
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
        ];

        assert_eq!(batch_groups, batch.groups);
        assert_eq!([8_usize, 0, 1, 0, 0, 0, 0, 0], batch.op_counts);
        assert_eq!(hasher::hash_elements(&batch_groups[..4]), hash);

        // --- push at the end of the 7th group overflows to the next batch -----------------------
        let ops = vec![
            Operation::Add,
            Operation::Mul,
            Operation::Push(Felt::new(1)),
            Operation::Push(Felt::new(2)),
            Operation::Push(Felt::new(3)),
            Operation::Push(Felt::new(4)),
            Operation::Push(Felt::new(5)),
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Mul,
            Operation::Add,
            Operation::Mul,
            Operation::Push(Felt::new(6)),
            Operation::Pad,
        ];

        let (batches, hash) = super::batch_ops(ops.clone());
        assert_eq!(2, batches.len());

        let batch0 = &batches[0];
        assert_eq!(ops[..17], batch0.ops);
        assert_eq!(7, batch0.num_groups());

        let batch0_groups = [
            build_group(&ops[..9]),
            Felt::new(1),
            Felt::new(2),
            Felt::new(3),
            Felt::new(4),
            Felt::new(5),
            build_group(&ops[9..17]),
            Felt::ZERO,
        ];

        assert_eq!(batch0_groups, batch0.groups);
        assert_eq!([9_usize, 0, 0, 0, 0, 0, 8, 0], batch0.op_counts);

        let batch1 = &batches[1];
        assert_eq!(ops[17..], batch1.ops);
        assert_eq!(2, batch1.num_groups());

        let batch1_groups = [
            build_group(&ops[17..]),
            Felt::new(6),
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
        ];
        assert_eq!(batch1_groups, batch1.groups);
        assert_eq!([2_usize, 0, 0, 0, 0, 0, 0, 0], batch1.op_counts);

        let all_groups = [batch0_groups, batch1_groups].concat();
        assert_eq!(hasher::hash_elements(&all_groups[..10]), hash);
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------

    fn build_group(ops: &[Operation]) -> Felt {
        let mut group = 0u64;
        for (i, op) in ops.iter().enumerate() {
            group |= (op.op_code() as u64) << (Operation::OP_BITS * i);
        }
        Felt::new(group)
    }
}
