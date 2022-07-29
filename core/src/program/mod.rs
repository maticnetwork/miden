use super::{
    hasher::{self, Digest},
    utils::{collections::Vec, Box},
    Felt, FieldElement, Operation,
};
use core::fmt;

pub mod blocks;
use blocks::CodeBlock;

mod library;
pub use library::Library;

// PROGRAM
// ================================================================================================
/// A program which can be executed by the VM.
///
/// A program is described by a Merkelized Abstract Syntax Tree (MAST), where each node is a
/// [CodeBlock]. Internal nodes describe control flow semantics of the program, while leaf nodes
/// contain linear sequences of instructions which contain no control flow.
#[derive(Clone, Debug)]
pub struct Program {
    root: CodeBlock,
}

impl Program {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Constructs a new program from the specified code block.
    pub fn new(root: CodeBlock) -> Self {
        Self { root }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the root code block of this program.
    pub fn root(&self) -> &CodeBlock {
        &self.root
    }

    /// Returns a hash of this program.
    pub fn hash(&self) -> Digest {
        self.root.hash()
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "begin {} end", self.root)
    }
}
