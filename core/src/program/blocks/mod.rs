use super::{hasher, Box, Digest, Felt, FieldElement, Operation, Vec};
use crate::DecoratorList;
use core::fmt;

mod call_block;
mod join_block;
mod loop_block;
mod proxy_block;
mod span_block;
mod split_block;

pub use call_block::Call;
pub use join_block::Join;
pub use loop_block::Loop;
pub use proxy_block::Proxy;
pub use span_block::{
    get_span_op_group_count, OpBatch, Span, BATCH_SIZE as OP_BATCH_SIZE,
    GROUP_SIZE as OP_GROUP_SIZE,
};
pub use split_block::Split;

// PROGRAM BLOCK
// ================================================================================================
/// TODO: add comments
#[derive(Clone, Debug)]
pub enum CodeBlock {
    Span(Span),
    Join(Join),
    Split(Split),
    Loop(Loop),
    Call(Call),
    Proxy(Proxy),
}

impl CodeBlock {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new Span block instantiated with the provided operations.
    pub fn new_span(operations: Vec<Operation>) -> Self {
        Self::Span(Span::new(operations))
    }

    /// Returns a new Span block instantiated with the provided operations and decorator list.
    pub fn new_span_with_decorators(operations: Vec<Operation>, decorators: DecoratorList) -> Self {
        Self::Span(Span::with_decorators(operations, decorators))
    }

    /// TODO: add comments
    pub fn new_join(blocks: [CodeBlock; 2]) -> Self {
        Self::Join(Join::new(blocks))
    }

    /// TODO: add comments
    pub fn new_split(t_branch: CodeBlock, f_branch: CodeBlock) -> Self {
        Self::Split(Split::new(t_branch, f_branch))
    }

    /// TODO: add comments
    pub fn new_loop(body: CodeBlock) -> Self {
        Self::Loop(Loop::new(body))
    }

    /// TODO: add comments
    pub fn new_call(code_hash: Digest) -> Self {
        Self::Call(Call::new(code_hash))
    }

    /// TODO: add comments
    pub fn new_proxy(code_hash: Digest) -> Self {
        Self::Proxy(Proxy::new(code_hash))
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns true if this code block is a [Span] block.
    pub fn is_span(&self) -> bool {
        matches!(self, CodeBlock::Span(_))
    }

    /// Returns a hash of this code block.
    pub fn hash(&self) -> Digest {
        match self {
            CodeBlock::Span(block) => block.hash(),
            CodeBlock::Join(block) => block.hash(),
            CodeBlock::Split(block) => block.hash(),
            CodeBlock::Loop(block) => block.hash(),
            CodeBlock::Call(block) => block.hash(),
            CodeBlock::Proxy(block) => block.hash(),
        }
    }
}

impl fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeBlock::Span(block) => write!(f, "{}", block),
            CodeBlock::Join(block) => write!(f, "{}", block),
            CodeBlock::Split(block) => write!(f, "{}", block),
            CodeBlock::Loop(block) => write!(f, "{}", block),
            CodeBlock::Call(block) => write!(f, "{}", block),
            CodeBlock::Proxy(block) => write!(f, "{}", block),
        }
    }
}
