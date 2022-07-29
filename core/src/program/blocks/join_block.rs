use super::{fmt, hasher, Box, CodeBlock, Digest};
use crate::{utils::collections::Vec, Decorator};

// JOIN BLOCKS
// ================================================================================================
/// A code block used to combine two other code blocks.
///
/// When the VM executes a Join block, it executes joined blocks in sequence one after the other.
///
/// Hash of a Join block is computed by hashing a concatenation of the hashes of joined blocks.
/// TODO: update hashing methodology to make it different from Split block.
#[derive(Clone, Debug)]
pub struct Join {
    body: Box<[CodeBlock; 2]>,
    hash: Digest,
    proc_markers: Vec<Decorator>,
}

impl Join {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [Join] block instantiated with the specified code blocks.
    pub fn new(body: [CodeBlock; 2]) -> Self {
        let hash = hasher::merge(&[body[0].hash(), body[1].hash()]);
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

    /// Returns a reference to the code block which is to be executed first when this join block
    /// is executed.
    pub fn first(&self) -> &CodeBlock {
        &self.body[0]
    }

    /// Returns a reference to the code block which is to be executed second when this join block
    /// is executed.
    pub fn second(&self) -> &CodeBlock {
        &self.body[1]
    }

    /// If a procedure starts at this join block, returns ProcMarker [Decorator] corresponding to
    /// the procedure. Returns None otherwise.
    pub fn proc_markers(&self) -> &[Decorator] {
        &self.proc_markers
    }

    /// If a procedure starts at this join block, adds ProcMarker [Decorator] corresponding the
    /// procedure to this join block.
    pub fn append_proc_marker(&mut self, proc_marker: Decorator) {
        self.proc_markers.push(proc_marker);
    }
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "join {} {} end", self.body[0], self.body[1])
    }
}
