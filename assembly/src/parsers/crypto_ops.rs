use super::{super::validate_operation, AssemblyError, Felt, Operation, Token, Vec};
use vm_core::{utils::PushMany, AdviceInjector, Decorator, DecoratorList};

// HASHING
// ================================================================================================
// The number of elements to be hashed by the rphash operation
const RPHASH_NUM_ELEMENTS: u64 = 8;

/// Appends RPPERM and stack manipulation operations to the span block as required to compute a
/// 2-to-1 Rescue Prime hash. The top of the stack is expected to be arranged with 2 words
/// (8 elements) to be hashed: [B, A, ...]. The resulting stack will contain the 2-to-1 hash result
/// [E, ...].
///
/// This assembly operation uses the VM operation RPPERM at its core, which permutes the top 12
/// elements of the stack.
///
/// To perform the operation, we do the following:
/// 1. Prepare the stack with 12 elements for RPPERM by pushing 4 more elements for the capacity,
///    including the number of elements to be hashed (8), so the stack looks like [C, B, A, ...]
///    where C is the capacity, and the number of elements is the deepest element in C.
/// 2. Reorder the stack so the capacity is deepest in the stack [B, A, C, ...]
/// 3. Append the RPPERM operation, which performs a Rescue Prime permutation on the top 12
///    elements and leaves an output of [F, E, D, ...] on the stack. E is our 2-to-1 hash result.
/// 4. Drop F and D to return our result [E, ...].
///
/// This operation takes 16 VM cycles.
///
/// # Errors
/// Returns an AssemblyError if:
/// - the operation is malformed.
/// - an unrecognized operation is received (anything other than rphash).
pub fn parse_rphash(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    validate_operation!(op, "rphash", 0);

    // Add 4 elements to the stack to prepare the capacity portion for the Rescue Prime permutation
    // The capacity should start at stack[8], and the number of elements to be hashed should
    // be deepest in the stack at stack[11]
    span_ops.push(Operation::Push(Felt::new(RPHASH_NUM_ELEMENTS)));
    span_ops.push_many(Operation::Pad, 3);
    span_ops.push(Operation::SwapW2);
    // restore the order of the top 2 words to be hashed
    span_ops.push(Operation::SwapW);

    // Do the Rescue Prime permutation on the top 12 elements in the stack
    span_ops.push(Operation::RpPerm);

    // Drop 4 elements (the part of the rate that doesn't have our result)
    span_ops.push_many(Operation::Drop, 4);

    // Move the top word (our result) down the stack
    span_ops.push(Operation::SwapW);

    // Drop 4 elements (the capacity portion)
    span_ops.push_many(Operation::Drop, 4);

    Ok(())
}

/// Appends an RPPERM operation to the span block, which performs a Rescue Prime permutation on the
/// top 12 elements of the stack.
///
/// This operation takes 1 VM cycle.
///
/// # Errors
/// Returns an AssemblyError if:
/// - the operation is malformed.
/// - an unrecognized operation is received (anything other than rpperm).
pub fn parse_rpperm(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    validate_operation!(op, "rpperm", 0);

    // append the machine op to the span block
    span_ops.push(Operation::RpPerm);

    Ok(())
}

// MERKLE TREES
// ================================================================================================

/// Parses the type of Merkle tree operation and appends a VM crypto operation and the stack
/// manipulations required for correct execution of the specified mtree op.
/// - "mtree.get" verifies that a Merkle tree with root R opens to node V at depth d and index i.
///   It uses the MPVERIFY operation in the processor.
/// - "mtree.set" updates a node in the Merkle tree with root R at depth d and index i to value V.
///   It uses the MRUPDATE operation with the parameter set to "false" so the old advice set is not
///   saved.
/// - "mtree.cwm" copies a Merkle tree with root R and updates a node at depth d and index i to
///   value V. It uses the MRUPDATE operation with the parameter set to "true" so the old advice
///   set is preserved.
///
/// # Errors:
/// Returns an AssemblyError if:
/// - the operation is malformed.
/// - an unrecognized operation is received (anything other than "mtree" with a valid variant of
///   "get", "set", or "cwm").
pub fn parse_mtree(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    decorators: &mut DecoratorList,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "mtree.cwm|get|set", 0);

    match op.parts()[1] {
        "get" => mtree_get(span_ops, decorators),
        "set" => mtree_set(span_ops, decorators),
        "cwm" => mtree_cwm(span_ops, decorators),
        _ => return Err(AssemblyError::invalid_op(op)),
    }

    Ok(())
}

/// Appends the MPVERIFY op and stack manipulations to the span block as required to verify that a
/// Merkle tree with root R opens to node V at depth d and index i. The stack is expected to be
/// arranged as follows (from the top):
/// - depth of the node, 1 element
/// - index of the node, 1 element
/// - current root of the tree, 4 elements
///
/// After the operations are executed, the stack will be arranged as follows:
/// - node V, 4 elements
/// - root of the tree, 4 elements
///
/// This operation takes 12 VM cycles.
fn mtree_get(span_ops: &mut Vec<Operation>, decorators: &mut DecoratorList) {
    // stack: [d, i, R, ...]
    // inject the node value we're looking for at the head of the advice tape
    decorators.push((
        span_ops.len(),
        Decorator::Advice(AdviceInjector::MerkleNode),
    ));

    // temporarily move d and i out of the way to make future stack manipulations easier
    // => [0, 0, d, i, R, ...]
    span_ops.push(Operation::Pad);
    span_ops.push(Operation::Pad);

    // read node value from advice tape => [V, 0, 0, d, i, R, ...]
    span_ops.push_many(Operation::Read, 4);

    // move d, i back to the top of the stack => [d, i, V, R, ...]
    span_ops.push(Operation::SwapW);
    span_ops.push(Operation::Drop);
    span_ops.push(Operation::Drop);

    // verify the node V for root R with depth d and index i
    // => [d, i, V, R, ...]
    span_ops.push(Operation::MpVerify);

    // drop d, i since they're no longer needed => [V, R, ...]
    span_ops.push(Operation::Drop);
    span_ops.push(Operation::Drop);
}

/// Appends the MRUPDATE op with a parameter of "false" and stack manipulations to the span block
/// as required to update a node in the Merkle tree with root R at depth d and index i to value V.
/// The stack is expected to be arranged as follows (from the top):
/// - depth of the node, 1 element
/// - index of the node, 1 element
/// - new value of the node, 4 element
/// - current root of the tree, 4 elements
///
/// After the operations are executed, the stack will be arranged as follows:
/// - new value of the node, 4 elements
/// - new root of the tree after the update, 4 elements
///
/// This operation takes 38 VM cycles.
fn mtree_set(span_ops: &mut Vec<Operation>, decorators: &mut DecoratorList) {
    // Duplicate the new value and reorder the stack as required for the call to MRUPDATE.
    // [d, i, V_new, R, ...] => [d, i, V_old, V_new, R, V_new_0, V_new_1] (overflowed)
    prep_stack_for_mrupdate(span_ops, decorators);

    // Update the Merkle tree with the new value without copying the old tree. This replaces the
    // old and new node values with the computed and new Merkle roots, respectively.
    // => [d, i, R_computed, R_new, R, V_new_0, V_new_1] (overflowed)
    span_ops.push(Operation::MrUpdate(false));

    // Validate that the computed root and the old root are equal and
    // drop values that are no longer needed (d, i, duplicate of old root).
    // => [R, R_new, V_new, ...]
    validate_root_after_mrupdate(span_ops);

    // drop the old root => [R_new, V_new, ...]
    span_ops.push_many(Operation::Drop, 4);

    // move the new value to the top of the stack => [V_new, R_new, ...]
    span_ops.push(Operation::SwapW);
}

/// Appends the MRUPDATE op with a parameter of "true" and stack manipulations to the span block as
/// required to copy a Merkle tree with root R and update the node in the copied tree at depth d
/// and index i to value V. The stack is expected to be arranged as follows (from the top):
/// - depth of the node, 1 element; this is expected to be the depth of the Merkle tree
/// - index of the node, 1 element
/// - new value of the node, 4 element
/// - current root of the tree, 4 elements
///
/// After the operations are executed, the stack will be arranged as follows:
/// - new value of the node V, 4 elements
/// - root of the new tree with the updated node value, 4 elements
/// - root of the old tree which was copied, 4 elements
///
/// This operation takes 34 VM cycles.
fn mtree_cwm(span_ops: &mut Vec<Operation>, decorators: &mut DecoratorList) {
    // Duplicate the new value and reorder the stack as required for the call to MRUPDATE.
    // [d, i, V_new, R, ...] => [d, i, V_old, V_new, R, V_new_0, V_new_1] (overflowed)
    prep_stack_for_mrupdate(span_ops, decorators);

    // update the Merkle tree with the new value and copy the old tree. This replaces the
    // old and new node values with the computed and new Merkle roots, respectively.
    // => [d, i, R_computed, R_new, R, V_new_0, V_new_1] (overflowed)
    span_ops.push(Operation::MrUpdate(true));

    // validate the computed root and the old root are equal and drop values no longer needed
    // => [R, R_new, V_new, ...]
    validate_root_after_mrupdate(span_ops);

    // move the new value to the top of the stack => [V_new, R_new, R, ...]
    span_ops.push(Operation::SwapW2);
}

/// Validates that two 4 word Merkle roots at the top of the stack are equal, then drops the
/// duplicate. The stack is expected to be arranged as follows (from the top):
/// - root of a Merkle tree, 4 elements
/// - root of a Merkle tree, 4 elements
///
/// This operation takes 20 VM cycles.
fn validate_and_drop_root(span_ops: &mut Vec<Operation>) {
    // verify the provided root and the computed root are equal
    span_ops.push(Operation::Dup7);
    span_ops.push(Operation::Dup4);
    span_ops.push(Operation::Eq);

    span_ops.push(Operation::Dup7);
    span_ops.push(Operation::Dup4);
    span_ops.push(Operation::Eq);
    span_ops.push(Operation::And);

    span_ops.push(Operation::Dup6);
    span_ops.push(Operation::Dup3);
    span_ops.push(Operation::Eq);
    span_ops.push(Operation::And);

    span_ops.push(Operation::Dup5);
    span_ops.push(Operation::Dup2);
    span_ops.push(Operation::Eq);
    span_ops.push(Operation::And);
    span_ops.push(Operation::Assert);

    // drop one of the duplicate roots
    span_ops.push_many(Operation::Drop, 4);
}

/// This is a helper function for assembly operations that update the Merkle tree. It preserves the
/// new node value so it can be left on the stack at the end of the assembly sequence and prepares
/// the stack with the elements and ordering expected by the VM's MRUPDATE operation. The stack is
/// expected to be arranged as follows (from the top):
/// - depth of the node, 1 element
/// - index of the node, 1 element
/// - new value of the node, 4 elements
/// - root of the Merkle tree, 4 elements
///
/// After the operations are executed, the stack will be arranged as follows:
/// - depth of the node, 1 element
/// - index of the node, 1 element
/// - old value of the node, 4 elements
/// - new value of the node, 4 elements
/// - root of the Merkle tree, 4 elements
/// - copy of the new value of the node, 4 elements
///
/// This operation takes 22 VM cycles.
fn prep_stack_for_mrupdate(span_ops: &mut Vec<Operation>, decorators: &mut DecoratorList) {
    // stack: [d, i, V_new, R, ...]

    // temporarily add two zeroes to the top of the stack to create word with d and i to make
    // future stack manipulations easier
    // => [0, 0, d, i, V_new, R, ...]
    span_ops.push(Operation::Pad);
    span_ops.push(Operation::Pad);

    // copy the new node value for use in the MRUPDATE op => [V_new, 0, 0, d, i, V_new, R, ...]
    span_ops.push_many(Operation::Dup7, 4);

    // move d, i and R to the top of the stack => [0, 0, d, i, R, V_new, V_new, ...]
    span_ops.push(Operation::SwapW3);
    span_ops.push(Operation::SwapW);

    // drop excess zeroes to be able to inject the node value to the advice tape
    // => [d, i, R, V_new, V_new, ...]
    span_ops.push(Operation::Drop);
    span_ops.push(Operation::Drop);

    // inject the node value we're looking for at the head of the advice tape
    decorators.push((
        span_ops.len(),
        Decorator::Advice(AdviceInjector::MerkleNode),
    ));
    // temporarily add two zeroes to the top of the stack again
    // => [0, 0, d, i, R, V_new, V_new, ...]
    span_ops.push(Operation::Pad);
    span_ops.push(Operation::Pad);

    // read old node value from advice tape => [V_old, 0, 0, d, i, R, V_new, V_new, ...]
    span_ops.push_many(Operation::Read, 4);

    // create the required order of elements => [0, 0, d, i, V_old, V_new, R, V_new, ...]
    span_ops.push(Operation::SwapW);
    span_ops.push(Operation::SwapDW);
    span_ops.push(Operation::SwapW);
    span_ops.push(Operation::SwapDW);

    // drop excess zeroes => [d, i, V_old, V_new, R, V_new, ...]
    span_ops.push(Operation::Drop);
    span_ops.push(Operation::Drop);
}

/// This is a helper function for assembly operations that update the Merkle tree. It validates
/// that the original and computed Merkle roots are equal and drops the values that are no longer
/// needed (depth, index, and the duplicate Merkle root). The stack is expected to be arranged as
/// follows (from the top):
/// - depth of the node, 1 element
/// - index of the node, 1 element
/// - computed root of the old Merkle tree, 4 elements
/// - new Merkle tree root, 4 elements
/// - old Merkle tree root, 4 elements
///
/// After the operations are executed, the stack will be arranged as follows:
/// - old Merkle tree root, 4 elements
/// - new Merkle tree root, 4 elements
///
/// This operation takes 24 VM cycles.
fn validate_root_after_mrupdate(span_ops: &mut Vec<Operation>) {
    // drop d, i => [R_computed, R_new, R, ...]
    span_ops.push(Operation::Drop);
    span_ops.push(Operation::Drop);

    // reorder the stack to prepare for comparing the computed and old roots
    // => [R_new, R_computed, R, ...]
    span_ops.push(Operation::SwapW);
    // => [R, R_computed, R_new, ...]
    span_ops.push(Operation::SwapW2);

    // validates the top 2 Merkle roots are equal and drops one copy of the old root
    // => [R, R_new, ...]
    validate_and_drop_root(span_ops);
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpperm() {
        let mut span_ops: Vec<Operation> = Vec::new();
        let op = Token::new("rpperm", 0);
        let expected = vec![Operation::RpPerm];

        parse_rpperm(&mut span_ops, &op).expect("Failed to parse rpperm");

        assert_eq!(span_ops, expected);
    }

    #[test]
    fn rpperm_invalid() {
        // parse_rpperm should return an error if called with an invalid or incorrect operation
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_pos = 0;

        let op_too_long = Token::new("rpperm.12", op_pos);
        let expected = AssemblyError::extra_param(&op_too_long);
        assert_eq!(
            parse_rpperm(&mut span_ops, &op_too_long).unwrap_err(),
            expected
        );

        let op_mismatch = Token::new("rphash", op_pos);
        let expected = AssemblyError::unexpected_token(&op_mismatch, "rpperm");
        assert_eq!(
            parse_rpperm(&mut span_ops, &op_mismatch).unwrap_err(),
            expected
        );
    }

    #[test]
    fn rphash_invalid() {
        // parse_rphash should return an error if called with an invalid or incorrect operation
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_pos = 0;

        let op_too_long = Token::new("rphash.12", op_pos);
        let expected = AssemblyError::extra_param(&op_too_long);
        assert_eq!(
            parse_rphash(&mut span_ops, &op_too_long).unwrap_err(),
            expected
        );

        let op_mismatch = Token::new("rpperm", op_pos);
        let expected = AssemblyError::unexpected_token(&op_mismatch, "rphash");
        assert_eq!(
            parse_rphash(&mut span_ops, &op_mismatch).unwrap_err(),
            expected
        );
    }

    #[test]
    fn mtree_invalid() {
        // parse_mtree should return an error if called with an invalid or incorrect operation
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_pos = 0;

        let op_too_short = Token::new("mtree", op_pos);
        let expected = AssemblyError::invalid_op(&op_too_short);
        let mut decorators = DecoratorList::new();
        assert_eq!(
            parse_mtree(&mut span_ops, &op_too_short, &mut decorators).unwrap_err(),
            expected
        );

        let op_too_long = Token::new("mtree.get.12", op_pos);
        let expected = AssemblyError::extra_param(&op_too_long);
        assert_eq!(
            parse_mtree(&mut span_ops, &op_too_long, &mut decorators).unwrap_err(),
            expected
        );

        let op_mismatch = Token::new("rpperm.get", op_pos);
        let expected = AssemblyError::unexpected_token(&op_mismatch, "mtree.cwm|get|set");
        assert_eq!(
            parse_mtree(&mut span_ops, &op_mismatch, &mut decorators).unwrap_err(),
            expected
        );
    }
}
