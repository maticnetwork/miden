use super::{fmt, hasher, Box, CodeBlock, Digest};
use crate::{utils::collections::Vec, Decorator};

// LOOP BLOCK
// ================================================================================================
/// A code block used to describe condition-based iterative execution.
///
/// When the VM encounters a Loop block, executes the loop's body if the top of the stack is `1`,
/// and skips the block if the top of the stack is `0`. If the top of the stack is neither `0` nor
/// `1`, the program fails. Once the loop body is fully executed, the VM checks the top of the
/// stack again. If the top of the stack is `1`, the loop is executed again, if it is `0`, the VM
/// stops executing the loop and moves to the next block. Thus, the body of the loop is executed
/// while the top of the stack remains `1` at the end of each loop iteration.
///
/// Hash of a Loop block is computed by hashing a concatenation of the loop's body hash with zero.
#[derive(Clone, Debug)]
pub struct Loop {
    body: Box<CodeBlock>,
    hash: Digest,
    proc_markers: Vec<Decorator>,
}

impl Loop {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [Loop] bock instantiated with the specified body.
    pub fn new(body: CodeBlock) -> Self {
        let hash = hasher::merge(&[body.hash(), Digest::default()]);
        Self {
            body: Box::new(body),
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

    /// Returns a reference to the code block which represents the body of the loop.
    pub fn body(&self) -> &CodeBlock {
        &self.body
    }

    /// If a procedure starts at this loop block, returns ProcMarker [Decorator] corresponding to
    /// the procedure. Returns None otherwise.
    pub fn proc_markers(&self) -> &[Decorator] {
        &self.proc_markers
    }

    /// If a procedure starts at this loop block, adds ProcMarker [Decorator] corresponding the
    /// procedure to this loop block.
    pub fn append_proc_marker(&mut self, proc_marker: Decorator) {
        self.proc_markers.push(proc_marker);
    }
}

impl fmt::Display for Loop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while.true {} end", self.body)
    }
}
