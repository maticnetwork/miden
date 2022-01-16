use vm_core::AdviceInjector;

use super::{validate_op_len, AssemblyError, BaseElement, Operation, Token};

// HASHING
// ================================================================================================
// The number of elements to be hashed by the rphash operation
const RPHASH_NUM_ELEMENTS: u64 = 8;

/// Appends RPPERM and stack manipulation operations to the span block as required to compute a
/// 2-to-1 Rescue Prime hash. The top of the stack is expected to be arranged with 2 words
/// (8 elements) to be hashed: [B, A, ...].
///
/// This assembly operation uses the VM operation RPPERM at its core, which permutes the top 12
/// elements of the stack.
///
/// To perform the operation, we do the following:
/// 1. Prepare the stack with 12 elements for RPPERM by pushing 4 more elements, the last of which
///    must equal the number of elements to be hashed (8).
/// 2. Append the RPPERM operation, which performs a Rescue Prime permutation on the top 12
///    elements and leaves an output of [E, D, C, ...] on the stack. E is our 2-to-1 hash result.
/// 3. Prepare to drop D and C by moving E further down the stack. This can be achieved by
///    swapping E and C with the SWAPW2 operation.
/// 4. Drop the top 8 elements from the stack, leaving our hash result at the top: [E, ...].
///
/// # Errors
/// Returns an AssemblyError if:
/// - the operation is malformed.
/// - an unrecognized operation is received (anything other than rphash).
pub fn parse_rphash(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    // validate the operation
    validate_op_len(op, 1, 0, 0)?;
    if op.parts()[0] != "rphash" {
        return Err(AssemblyError::unexpected_token(op, "rphash"));
    }

    // Add 4 elements to the stack to prepare for the Rescue Prime permutation
    // The element on top of the stack should be the number of elements to be hashed
    for _ in 0..3 {
        span_ops.push(Operation::Pad);
    }
    span_ops.push(Operation::Push(BaseElement::new(RPHASH_NUM_ELEMENTS)));

    // Do the Rescue Prime permutation on the top 12 elements in the stack
    span_ops.push(Operation::RpPerm);

    // Swap the top word (our result) with the 3rd word so we can easily drop words 2 and 3
    span_ops.push(Operation::SwapW2);

    // Drop 8 elements
    for _ in 0..8 {
        span_ops.push(Operation::Drop);
    }

    Ok(())
}

/// Appends an RPPERM operation to the span block, which performs a Rescue Prime permutation on the
/// top 12 elements of the stack.
///
/// # Errors
/// Returns an AssemblyError if:
/// - the operation is malformed.
/// - an unrecognized operation is received (anything other than rpperm).
pub fn parse_rpperm(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    // validate the operation
    validate_op_len(op, 1, 0, 0)?;
    if op.parts()[0] != "rpperm" {
        return Err(AssemblyError::unexpected_token(op, "rpperm"));
    }

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
pub fn parse_mtree(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    // validate operation
    validate_op_len(op, 2, 0, 0)?;
    if op.parts()[0] != "mtree" {
        return Err(AssemblyError::unexpected_token(op, "mtree.{get|set|cwm}"));
    }

    match op.parts()[1] {
        "get" => mtree_get(span_ops),
        "set" => mtree_set(span_ops),
        "cwm" => mtree_cwm(span_ops),
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
fn mtree_get(span_ops: &mut Vec<Operation>) {
    // stack: [d, i, R, ...]
    // inject the node value we're looking for at the head of the advice tape
    span_ops.push(Operation::Advice(AdviceInjector::MerkleNode));

    // temporarily move d and i out of the way to make future stack manipulations easier
    // => [R, d, i, ...]
    span_ops.push(Operation::MovDn5);
    span_ops.push(Operation::MovDn5);

    // read node value from advice tape => [V, R, d, i, ...]
    for _ in 0..4 {
        span_ops.push(Operation::Read);
    }

    // Duplicate the node value at the top of the stack and save a copy deeper in the stack. This
    // allows the new copy of the node to be used in MPVERIFY and keeps a copy to return at the end
    // swap root and node => [R, V, d, i, ...]
    span_ops.push(Operation::SwapW);
    // copy the node value for use in MPVERIFY => [V, R, V, d, i ...]
    for _ in 0..4 {
        span_ops.push(Operation::Dup7);
    }

    // move d, i back to the top of the stack => [d, i, V, R, V, ...]
    span_ops.push(Operation::MovUp13);
    span_ops.push(Operation::MovUp13);

    // verify the node V for root R with depth d and index i
    // => [d, i, R_computed, R, V, ...] where R_computed is the computed root for node V at d, i
    span_ops.push(Operation::MpVerify);

    // drop d, i since they're no longer needed => [R_computed, R, V, ...]
    span_ops.push(Operation::Drop);
    span_ops.push(Operation::Drop);

    // verify that the computed root for node V equals the provided root, then drop the duplicate
    // => [R, V, ...]
    validate_and_drop_root(span_ops);

    // move the retrieved & verified node value to the top of the stack => [V, R, ...]
    span_ops.push(Operation::SwapW);
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
fn mtree_set(span_ops: &mut Vec<Operation>) {
    // Duplicate the new value and reorder the stack as required for the call to MRUPDATE.
    // [d, i, V_new, R, ...] => [d, i, V_old, V_new, R, V_new_0, V_new_1] (overflowed)
    prep_stack_for_mrupdate(span_ops);

    // Update the Merkle tree with the new value without copying the old tree. This replaces the
    // old and new node values with the computed and new Merkle roots, respectively.
    // => [d, i, R_computed, R_new, R, V_new_0, V_new_1] (overflowed)
    span_ops.push(Operation::MrUpdate(false));

    // Validate that the computed root and the old root are equal and
    // drop values that are no longer needed (d, i, duplicate of old root).
    // => [R, R_new, V_new, ...]
    validate_root_after_mrupdate(span_ops);

    // drop the old root => [R_new, V_new, ...]
    for _ in 0..4 {
        span_ops.push(Operation::Drop);
    }

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
fn mtree_cwm(span_ops: &mut Vec<Operation>) {
    // Duplicate the new value and reorder the stack as required for the call to MRUPDATE.
    // [d, i, V_new, R, ...] => [d, i, V_old, V_new, R, V_new_0, V_new_1] (overflowed)
    prep_stack_for_mrupdate(span_ops);

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
fn validate_and_drop_root(span_ops: &mut Vec<Operation>) {
    // verify the provided root and the computed root are equal
    span_ops.push(Operation::Eqw);
    span_ops.push(Operation::Assert);

    // drop one of the duplicate roots
    for _ in 0..4 {
        span_ops.push(Operation::Drop);
    }
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
fn prep_stack_for_mrupdate(span_ops: &mut Vec<Operation>) {
    // stack: [d, i, V_new, R, ...]
    // temporarily move d and i out of the way to make future stack manipulations easier
    // => [V_new, R, d, i, ...]
    span_ops.push(Operation::MovDn9);
    span_ops.push(Operation::MovDn9);

    // move the root R to the top of the stack to prepare for reading from the advice set
    // => [R, V_new, d, i, ...]
    span_ops.push(Operation::SwapW);

    // move d, i back to the top of the stack => [d, i, R, V_new, ...]
    span_ops.push(Operation::MovUp9);
    span_ops.push(Operation::MovUp9);

    // inject the node value we're looking for at the head of the advice tape
    span_ops.push(Operation::Advice(AdviceInjector::MerkleNode));

    // copy the new node value for use in the MRUPDATE op => [V_new, d, i, R, V_new, ...]
    for _ in 0..4 {
        span_ops.push(Operation::Dup9);
    }

    // read old node value from advice tape => [V_old, V_new, d, i,  R, V_new_0, V_new_1] (overflow)
    for _ in 0..4 {
        span_ops.push(Operation::Read);
    }

    // move d, i to the top of the stack => [d, i, V_old, V_new, R, V_new_0, V_new_1]
    span_ops.push(Operation::MovUp9);
    span_ops.push(Operation::MovUp9);
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
    fn rphash() {
        // adds a word to the stack specifying the number of elements to be hashed (8)
        // does a rescue prime permutation
        // keeps the top word as the result but drops the other 8 elements
        let mut span_ops: Vec<Operation> = Vec::new();
        let op = Token::new("rphash", 0);

        // state of stack before permutation
        let mut expected = vec![
            Operation::Pad,
            Operation::Pad,
            Operation::Pad,
            Operation::Push(BaseElement::new(8)),
        ];
        // rp permutation leaves stack as [A, B, C,...]
        expected.push(Operation::RpPerm);
        // swap A and C, since A is the result we want --> gives [C, B, A, ...]
        expected.push(Operation::SwapW2);
        // drop C, B
        let drop8 = vec![Operation::Drop; 8];
        expected.extend_from_slice(&drop8);

        parse_rphash(&mut span_ops, &op).expect("Failed to parse rphash");
        assert_eq!(span_ops, expected);
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
        assert_eq!(
            parse_mtree(&mut span_ops, &op_too_short).unwrap_err(),
            expected
        );

        let op_too_long = Token::new("mtree.get.12", op_pos);
        let expected = AssemblyError::extra_param(&op_too_long);
        assert_eq!(
            parse_mtree(&mut span_ops, &op_too_long).unwrap_err(),
            expected
        );

        let op_mismatch = Token::new("rpperm.get", op_pos);
        let expected = AssemblyError::unexpected_token(&op_mismatch, "mtree.{get|set|cwm}");
        assert_eq!(
            parse_mtree(&mut span_ops, &op_mismatch).unwrap_err(),
            expected
        );
    }
}
