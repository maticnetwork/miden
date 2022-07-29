use super::{
    hasher::{self, Digest},
    AdviceSetError, Felt, FieldElement, Word,
};
use crate::utils::collections::Vec;
use core::slice;
use math::log2;
use winter_utils::uninit_vector;

// MERKLE TREE
// ================================================================================================

/// A fully-balanced binary Merkle tree (i.e., a tree where the number of leaves is a power of two).
///
/// This struct is intended to be used as one of the variants of the MerkleSet enum.
#[derive(Clone, Debug)]
pub struct MerkleTree {
    nodes: Vec<Word>,
}

impl MerkleTree {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a Merkle tree instantiated from the provided leaves.
    ///
    /// # Errors
    /// Returns an error if the number of leaves is smaller than two or is not a power of two.
    pub fn new(leaves: Vec<Word>) -> Result<Self, AdviceSetError> {
        let n = leaves.len();
        if n <= 1 {
            return Err(AdviceSetError::DepthTooSmall);
        } else if !n.is_power_of_two() {
            return Err(AdviceSetError::NumLeavesNotPowerOfTwo(n));
        }

        // create un-initialized vector to hold all tree nodes
        let mut nodes = unsafe { uninit_vector(2 * n) };
        nodes[0] = [Felt::ZERO; 4];

        // copy leaves into the second part of the nodes vector
        nodes[n..].copy_from_slice(&leaves);

        // re-interpret nodes as an array of two nodes fused together
        let two_nodes = unsafe { slice::from_raw_parts(nodes.as_ptr() as *const [Digest; 2], n) };

        // calculate all internal tree nodes
        for i in (1..n).rev() {
            nodes[i] = hasher::merge(&two_nodes[i]).into();
        }

        Ok(Self { nodes })
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the root of this Merkle tree.
    pub fn root(&self) -> Word {
        self.nodes[1]
    }

    /// Returns the depth of this Merkle tree.
    ///
    /// Merkle tree of depth 1 has two leaves, depth 2 has four leaves etc.
    pub fn depth(&self) -> u32 {
        log2(self.nodes.len() / 2)
    }

    /// Returns a node at the specified depth and index.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The specified depth is greater than the depth of the tree.
    /// * The specified index not valid for the specified depth.
    pub fn get_node(&self, depth: u32, index: u64) -> Result<Word, AdviceSetError> {
        if depth == 0 {
            return Err(AdviceSetError::DepthTooSmall);
        } else if depth > self.depth() {
            return Err(AdviceSetError::DepthTooBig(depth));
        }
        if index >= 2u64.pow(depth) {
            return Err(AdviceSetError::InvalidIndex(depth, index));
        }

        let pos = 2usize.pow(depth as u32) + (index as usize);
        Ok(self.nodes[pos])
    }

    /// Returns a Merkle path to the node at the specified depth and index. The node itself is
    /// not included in the path.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The specified depth is greater than the depth of the tree.
    /// * The specified index not valid for the specified depth.
    pub fn get_path(&self, depth: u32, index: u64) -> Result<Vec<Word>, AdviceSetError> {
        if depth == 0 {
            return Err(AdviceSetError::DepthTooSmall);
        } else if depth > self.depth() {
            return Err(AdviceSetError::DepthTooBig(depth));
        }
        if index >= 2u64.pow(depth) {
            return Err(AdviceSetError::InvalidIndex(depth, index));
        }

        let mut path = Vec::with_capacity(depth as usize);
        let mut pos = 2usize.pow(depth as u32) + (index as usize);

        while pos > 1 {
            path.push(self.nodes[pos ^ 1]);
            pos >>= 1;
        }

        Ok(path)
    }

    /// Replaces the leaf at the specified index with the provided value.
    ///
    /// # Errors
    /// Returns an error if the specified index is not a valid leaf index for this tree.
    pub fn update_leaf(&mut self, index: u64, value: Word) -> Result<(), AdviceSetError> {
        let depth = self.depth();
        if index >= 2u64.pow(depth) {
            return Err(AdviceSetError::InvalidIndex(depth, index));
        }

        let mut index = 2usize.pow(depth) + index as usize;
        self.nodes[index] = value;

        let n = self.nodes.len() / 2;
        let two_nodes =
            unsafe { slice::from_raw_parts(self.nodes.as_ptr() as *const [Digest; 2], n) };

        for _ in 0..depth {
            index /= 2;
            self.nodes[index] = hasher::merge(&two_nodes[index]).into();
        }

        Ok(())
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{Felt, FieldElement, Word};
    use crypto::{hashers::Rp64_256, ElementHasher, Hasher};

    const LEAVES4: [Word; 4] = [
        int_to_node(1),
        int_to_node(2),
        int_to_node(3),
        int_to_node(4),
    ];

    const LEAVES8: [Word; 8] = [
        int_to_node(1),
        int_to_node(2),
        int_to_node(3),
        int_to_node(4),
        int_to_node(5),
        int_to_node(6),
        int_to_node(7),
        int_to_node(8),
    ];

    #[test]
    fn build_merkle_tree() {
        let tree = super::MerkleTree::new(LEAVES4.to_vec()).unwrap();
        assert_eq!(8, tree.nodes.len());

        // leaves were copied correctly
        for (a, b) in tree.nodes.iter().skip(4).zip(LEAVES4.iter()) {
            assert_eq!(a, b);
        }

        let (root, node2, node3) = compute_internal_nodes();

        assert_eq!(root, tree.nodes[1]);
        assert_eq!(node2, tree.nodes[2]);
        assert_eq!(node3, tree.nodes[3]);

        assert_eq!(root, tree.root());
    }

    #[test]
    fn get_leaf() {
        let tree = super::MerkleTree::new(LEAVES4.to_vec()).unwrap();

        // check depth 2
        assert_eq!(LEAVES4[0], tree.get_node(2, 0).unwrap());
        assert_eq!(LEAVES4[1], tree.get_node(2, 1).unwrap());
        assert_eq!(LEAVES4[2], tree.get_node(2, 2).unwrap());
        assert_eq!(LEAVES4[3], tree.get_node(2, 3).unwrap());

        // check depth 1
        let (_, node2, node3) = compute_internal_nodes();

        assert_eq!(node2, tree.get_node(1, 0).unwrap());
        assert_eq!(node3, tree.get_node(1, 1).unwrap());
    }

    #[test]
    fn get_path() {
        let tree = super::MerkleTree::new(LEAVES4.to_vec()).unwrap();

        let (_, node2, node3) = compute_internal_nodes();

        // check depth 2
        assert_eq!(vec![LEAVES4[1], node3], tree.get_path(2, 0).unwrap());
        assert_eq!(vec![LEAVES4[0], node3], tree.get_path(2, 1).unwrap());
        assert_eq!(vec![LEAVES4[3], node2], tree.get_path(2, 2).unwrap());
        assert_eq!(vec![LEAVES4[2], node2], tree.get_path(2, 3).unwrap());

        // check depth 1
        assert_eq!(vec![node3], tree.get_path(1, 0).unwrap());
        assert_eq!(vec![node2], tree.get_path(1, 1).unwrap());
    }

    #[test]
    fn update_leaf() {
        let mut tree = super::MerkleTree::new(LEAVES8.to_vec()).unwrap();

        // update one leaf
        let index = 3;
        let new_node = int_to_node(9);
        let mut expected_leaves = LEAVES8.to_vec();
        expected_leaves[index as usize] = new_node;
        let expected_tree = super::MerkleTree::new(expected_leaves.clone()).unwrap();

        tree.update_leaf(index, new_node).unwrap();
        assert_eq!(expected_tree.nodes, tree.nodes);

        // update another leaf
        let index = 6;
        let new_node = int_to_node(10);
        expected_leaves[index as usize] = new_node;
        let expected_tree = super::MerkleTree::new(expected_leaves.clone()).unwrap();

        tree.update_leaf(index, new_node).unwrap();
        assert_eq!(expected_tree.nodes, tree.nodes);
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    fn compute_internal_nodes() -> (Word, Word, Word) {
        let node2 = Rp64_256::hash_elements(&[LEAVES4[0], LEAVES4[1]].concat());
        let node3 = Rp64_256::hash_elements(&[LEAVES4[2], LEAVES4[3]].concat());
        let root = Rp64_256::merge(&[node2, node3]);

        (root.into(), node2.into(), node3.into())
    }

    const fn int_to_node(value: u64) -> Word {
        [Felt::new(value), Felt::ZERO, Felt::ZERO, Felt::ZERO]
    }
}
