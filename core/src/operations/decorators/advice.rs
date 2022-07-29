use core::fmt;

/// TODO: add docs
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AdviceInjector {
    /// Injects a node of the Merkle tree specified by the values on the stack at the head of the
    /// advice tape. The stack is expected to be arranged as follows (from the top):
    /// - depth of the node, 1 element
    /// - index of the node, 1 element
    /// - root of the tree, 4 elements
    MerkleNode,

    /// Injects the result of u64 division (both the quotient and the remainder) at the head of
    /// the advice tape. The stack is expected to be arranged as follows (from the top):
    /// - divisor split into two 32-bit elements
    /// - dividend split into two 32-bit elements
    ///
    /// The result is injected into the advice tape as follows: first the remainder is injected,
    /// then the quotient is injected.
    DivResultU64,
}

impl fmt::Display for AdviceInjector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MerkleNode => write!(f, "merkle_node"),
            Self::DivResultU64 => write!(f, "div_result_u64"),
        }
    }
}
