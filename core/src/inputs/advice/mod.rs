use super::{hasher, AdviceSetError, Felt, FieldElement, Word};
use crate::utils::collections::Vec;

mod merkle_tree;
use merkle_tree::MerkleTree;
mod merkle_path_set;
use merkle_path_set::MerklePathSet;
mod sparse_merkle_tree;
use sparse_merkle_tree::SparseMerkleTree;

// ADVICE SET
// ================================================================================================

/// TODO: add docs
#[derive(Clone, Debug)]
pub enum AdviceSet {
    MerkleTree(MerkleTree),
    SparseMerkleTree(SparseMerkleTree),
    MerklePathSet(MerklePathSet),
}

impl AdviceSet {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new [AdviceSet] instantiated as a Merkle tree from the provided leaves.
    ///
    /// # Errors
    /// Returns an error if the number of leaves is smaller than two or is not a power of two.
    pub fn new_merkle_tree(leaves: Vec<Word>) -> Result<Self, AdviceSetError> {
        // TODO: change the signature to accept a vector of [u8; 32]?
        Ok(Self::MerkleTree(MerkleTree::new(leaves)?))
    }

    /// Returns a new [AdviceSet] instantiated as a Sparse Merkle tree from the provided leaves.
    ///
    /// # Errors
    /// Returns an error if the number of leaves is smaller than two or is not a power of two.
    pub fn new_sparse_merkle_tree(
        keys: Vec<u64>,
        values: Vec<Word>,
        depth: u32,
    ) -> Result<Self, AdviceSetError> {
        Ok(Self::SparseMerkleTree(SparseMerkleTree::new(
            keys, values, depth,
        )?))
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a root of this advice set.
    pub fn root(&self) -> Word {
        match self {
            Self::MerkleTree(tree) => tree.root(),
            Self::SparseMerkleTree(tree) => tree.root(),
            Self::MerklePathSet(set) => set.root(),
        }
    }

    /// Returns the maximum depth of this advice set.
    pub fn depth(&self) -> u32 {
        match self {
            Self::MerkleTree(tree) => tree.depth(),
            Self::SparseMerkleTree(tree) => tree.depth(),
            Self::MerklePathSet(set) => set.depth(),
        }
    }

    /// Returns a node located at the specified depth and index.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The specified depth is greater than the depth of this advice set.
    /// - The specified index is invalid in the context of the specified depth.
    /// - This advice set does not contain a node at the specified index and depth.
    pub fn get_node(&self, depth: u32, index: u64) -> Result<Word, AdviceSetError> {
        match self {
            Self::MerkleTree(tree) => tree.get_node(depth, index),
            Self::SparseMerkleTree(tree) => tree.get_node(depth, index),
            Self::MerklePathSet(set) => set.get_node(depth, index),
        }
    }

    /// Returns a Merkle path to a node located at the specified depth and index. The node itself
    /// is not included in the path.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The specified depth is greater than the depth of this advice set.
    /// - The specified index is invalid in the context of the specified depth.
    /// - This advice set does not contain a node at the specified index and depth.
    pub fn get_path(&self, depth: u32, index: u64) -> Result<Vec<Word>, AdviceSetError> {
        match self {
            Self::MerkleTree(tree) => tree.get_path(depth, index),
            Self::SparseMerkleTree(tree) => tree.get_path(depth, index),
            Self::MerklePathSet(set) => set.get_path(depth, index),
        }
    }

    // DATA MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Replaces the leaf at the specified index with the provided value.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The specified index is not a valid leaf index for this advice set.
    /// - This advice set does not contain a leaf at the specified index.
    pub fn update_leaf(&mut self, index: u64, value: Word) -> Result<(), AdviceSetError> {
        match self {
            Self::MerkleTree(tree) => tree.update_leaf(index, value),
            Self::SparseMerkleTree(tree) => tree.update_leaf(index, value),
            Self::MerklePathSet(set) => set.update_leaf(index, value),
        }
    }
}
