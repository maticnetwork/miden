use super::{fmt, hasher, Box, CodeBlock, Digest};
use crate::{utils::collections::Vec, Decorator};

// SPLIT BLOCK
// ================================================================================================
/// A code block used to describe conditional execution.
///
/// When the VM executes a Split bock, either the true branch or the false branch of the block is
/// executed. Specifically, if the top of the stack is `1`, the true branch is executed, and if
/// the top of the stack is `0`, the false branch is executed. If the top of the stack is neither
/// `0` nor `1`, the program fails.
///
/// Hash of a Split block is computed by hashing a concatenation of the true and the false branch
/// hashes.
#[derive(Clone, Debug)]
pub struct Split {
    branches: Box<[CodeBlock; 2]>,
    hash: Digest,
    proc_markers: Vec<Decorator>,
}

impl Split {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [Split] block instantiated with the specified true and false branches.
    pub fn new(t_branch: CodeBlock, f_branch: CodeBlock) -> Self {
        let hash = hasher::merge(&[t_branch.hash(), f_branch.hash()]);
        Self {
            branches: Box::new([t_branch, f_branch]),
            hash,
            proc_markers: Vec::new(),
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a hash of this code block.
    pub fn hash(&self) -> Digest {
        self.hash
    }

    /// Returns a reference to the code block which is to be executed when the top of the stack
    /// is `1`.
    pub fn on_true(&self) -> &CodeBlock {
        &self.branches[0]
    }

    /// Returns a reference to the code block which is to be executed when the top of the stack
    /// is `0`.
    pub fn on_false(&self) -> &CodeBlock {
        &self.branches[1]
    }

    /// If a procedure starts at this split block, returns ProcMarker [Decorator] corresponding to
    /// the procedure. Returns None otherwise.
    pub fn proc_markers(&self) -> &[Decorator] {
        &self.proc_markers
    }

    /// If a procedure starts at this join block, adds ProcMarker [Decorator] corresponding the
    /// procedure to this split block.
    pub fn append_proc_marker(&mut self, proc_marker: Decorator) {
        self.proc_markers.push(proc_marker);
    }
}

impl fmt::Display for Split {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "if.true {} else {} end",
            self.branches[0], self.branches[1]
        )
    }
}
