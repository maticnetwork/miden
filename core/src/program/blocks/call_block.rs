use super::{fmt, hasher, Digest};

// CALL BLOCK
// ================================================================================================
/// A code block describing a function call.
///
/// When the VM executes a Call block, it simply executes the code of the underlying function.
/// Thus, to execute a function call, the VM must have access to the function's body, otherwise,
/// the execution fails.
///
/// Hash of a Call block is computed by hashing a concatenation of the function's body hash with
/// zero.
/// TODO: update hashing methodology to make it different from Loop block.
#[derive(Clone, Debug)]
pub struct Call {
    hash: Digest,
    fn_hash: Digest,
}

impl Call {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [Call] block instantiated with the specified function body hash.
    pub fn new(fn_hash: Digest) -> Self {
        let hash = hasher::merge(&[fn_hash, Digest::default()]);
        Self { hash, fn_hash }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a hash of this code block.
    pub fn hash(&self) -> Digest {
        self.hash
    }

    /// Returns a hash of the function to be called by this block.
    pub fn fn_hash(&self) -> Digest {
        self.fn_hash
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "call.{:?}", self.fn_hash) // TODO
    }
}
