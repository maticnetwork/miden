use super::{hasher, AdviceSetError, Felt, FieldElement, Word};
use crate::utils::collections::{BTreeMap, Vec};

// MERKLE PATH SET
// ================================================================================================

/// A set of Merkle paths.
///
/// This struct is intended to be used as one of the variants of the MerkleSet enum.
#[derive(Clone, Debug)]
pub struct MerklePathSet {
    root: Word,
    total_depth: u32,
    paths: BTreeMap<u64, Vec<Word>>,
}

impl MerklePathSet {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Returns an empty MerklePathSet.
    pub fn new(depth: u32) -> Result<Self, AdviceSetError> {
        let root = [Felt::ZERO; 4];
        let paths = BTreeMap::<u64, Vec<Word>>::new();

        Ok(Self {
            root,
            total_depth: depth,
            paths,
        })
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Adds the specified Merkle path to this [MerklePathSet]. The `index` and `value` parameters
    /// specify the leaf node at which the path starts.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The specified index is not valid in the context of this Merkle path set (i.e., the index
    ///   implies a greater depth than is specified for this set).
    /// - The specified path is not consistent with other paths in the set (i.e., resolves to a
    ///   different root).
    pub fn add_path(
        &mut self,
        index: u64,
        value: Word,
        path: Vec<Word>,
    ) -> Result<(), AdviceSetError> {
        let depth = (path.len() + 1) as u32;
        if depth != self.total_depth {
            return Err(AdviceSetError::InvalidDepth(self.total_depth, depth));
        }

        // Actual number of node in tree
        let pos = 2u64.pow(self.total_depth) + index;

        // Index of the leaf path in map. Paths of neighboring leaves are stored in one key-value pair
        let half_pos = (pos / 2) as u64;

        let mut extended_path = path;
        if is_even(pos) {
            extended_path.insert(0, value);
        } else {
            extended_path.insert(1, value);
        }

        let root_of_current_path = compute_path_root(&extended_path, depth, index);
        if self.root == [Felt::ZERO; 4] {
            self.root = root_of_current_path;
        } else if self.root != root_of_current_path {
            return Err(AdviceSetError::InvalidPath(extended_path));
        }
        self.paths.insert(half_pos, extended_path);

        Ok(())
    }

    /// Returns the root to which all paths in this set resolve.
    pub fn root(&self) -> Word {
        self.root
    }

    /// Returns the depth of the Merkle tree implied by the paths stored in this set.
    ///
    /// Merkle tree of depth 1 has two leaves, depth 2 has four leaves etc.
    pub fn depth(&self) -> u32 {
        self.total_depth
    }

    /// Returns a node at the specified index.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The specified index not valid for the depth of structure.
    /// * Requested node does not exist in the set.
    pub fn get_node(&self, depth: u32, index: u64) -> Result<Word, AdviceSetError> {
        if index >= 2u64.pow(self.total_depth) {
            return Err(AdviceSetError::InvalidIndex(self.total_depth, index));
        }
        if depth != self.total_depth {
            return Err(AdviceSetError::InvalidDepth(self.total_depth, depth));
        }

        let pos = 2u64.pow(depth) + index;
        let index = (pos / 2) as u64;

        match self.paths.get(&index) {
            None => Err(AdviceSetError::NodeNotInSet(index)),
            Some(path) => {
                if is_even(pos) {
                    Ok(path[0])
                } else {
                    Ok(path[1])
                }
            }
        }
    }

    /// Returns a Merkle path to the node at the specified index. The node itself is
    /// not included in the path.
    ///
    /// # Errors
    /// Returns an error if:
    /// * The specified index not valid for the depth of structure.
    /// * Node of the requested path does not exist in the set.
    pub fn get_path(&self, depth: u32, index: u64) -> Result<Vec<Word>, AdviceSetError> {
        if index >= 2u64.pow(self.total_depth) {
            return Err(AdviceSetError::InvalidIndex(self.total_depth, index));
        }
        if depth != self.total_depth {
            return Err(AdviceSetError::InvalidDepth(self.total_depth, depth));
        }

        let pos = 2u64.pow(depth) + index;
        let index = pos / 2;

        match self.paths.get(&index) {
            None => Err(AdviceSetError::NodeNotInSet(index)),
            Some(path) => {
                let mut local_path = path.clone();
                if is_even(pos) {
                    local_path.remove(0);
                    Ok(local_path)
                } else {
                    local_path.remove(1);
                    Ok(local_path)
                }
            }
        }
    }

    /// Replaces the leaf at the specified index with the provided value.
    ///
    /// # Errors
    /// Returns an error if:
    /// * Requested node does not exist in the set.
    pub fn update_leaf(&mut self, index: u64, value: Word) -> Result<(), AdviceSetError> {
        let depth = self.depth();
        if index >= 2u64.pow(depth) {
            return Err(AdviceSetError::InvalidIndex(depth, index));
        }
        let pos = 2u64.pow(depth) + index;

        let path = match self.paths.get_mut(&(pos / 2)) {
            None => return Err(AdviceSetError::NodeNotInSet(index)),
            Some(path) => path,
        };

        // Fill old_hashes vector -----------------------------------------------------------------
        let (old_hashes, _) = compute_path_trace(path, depth, index);

        // Fill new_hashes vector -----------------------------------------------------------------
        if is_even(pos) {
            path[0] = value;
        } else {
            path[1] = value;
        }

        let (new_hashes, new_root) = compute_path_trace(path, depth, index);
        self.root = new_root;

        // update paths ---------------------------------------------------------------------------
        for path in self.paths.values_mut() {
            for i in (0..old_hashes.len()).rev() {
                if path[i + 2] == old_hashes[i] {
                    path[i + 2] = new_hashes[i];
                    break;
                }
            }
        }

        Ok(())
    }
}

// HELPER FUNCTIONS
// --------------------------------------------------------------------------------------------

fn is_even(pos: u64) -> bool {
    pos & 1 == 0
}

/// Calculates the hash of the parent node by two sibling ones
/// - node — current node
/// - node_pos — position of the current node
/// - sibling — neighboring vertex in the tree
fn calculate_parent_hash(node: Word, node_pos: u64, sibling: Word) -> Word {
    if is_even(node_pos) {
        hasher::merge(&[node.into(), sibling.into()]).into()
    } else {
        hasher::merge(&[sibling.into(), node.into()]).into()
    }
}

/// Returns vector of hashes from current to the root
fn compute_path_trace(path: &[Word], depth: u32, index: u64) -> (Vec<Word>, Word) {
    let mut pos = 2u64.pow(depth) + index;

    let mut computed_hashes = Vec::<Word>::new();

    let mut comp_hash = hasher::merge(&[path[0].into(), path[1].into()]).into();

    if path.len() != 2 {
        for path_hash in path.iter().skip(2) {
            computed_hashes.push(comp_hash);
            pos /= 2;
            comp_hash = calculate_parent_hash(comp_hash, pos, *path_hash);
        }
    }

    (computed_hashes, comp_hash)
}

/// Returns hash of the root
fn compute_path_root(path: &[Word], depth: u32, index: u64) -> Word {
    let mut pos = 2u64.pow(depth) + index;

    // hash that is obtained after calculating the current hash and path hash
    let mut comp_hash = hasher::merge(&[path[0].into(), path[1].into()]).into();

    for path_hash in path.iter().skip(2) {
        pos /= 2;
        comp_hash = calculate_parent_hash(comp_hash, pos, *path_hash);
    }

    comp_hash
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use crate::inputs::advice::merkle_path_set::calculate_parent_hash;

    use super::super::{Felt, FieldElement, Word};

    #[test]
    fn add_and_get_path() {
        let path_6 = vec![int_to_node(7), int_to_node(45), int_to_node(123)];
        let hash_6 = int_to_node(6);
        let index = 6u64;
        let depth = 4u32;
        let mut set = super::MerklePathSet::new(depth).unwrap();

        set.add_path(index, hash_6, path_6.clone()).unwrap();
        let stored_path_6 = set.get_path(depth, index).unwrap();

        assert_eq!(path_6, stored_path_6);
        assert!(set.get_path(depth, 15u64).is_err())
    }

    #[test]
    fn get_node() {
        let path_6 = vec![int_to_node(7), int_to_node(45), int_to_node(123)];
        let hash_6 = int_to_node(6);
        let index = 6u64;
        let depth = 4u32;
        let mut set = super::MerklePathSet::new(depth).unwrap();

        set.add_path(index, hash_6, path_6).unwrap();

        assert_eq!(int_to_node(6u64), set.get_node(depth, index).unwrap());
        assert!(set.get_node(depth, 15u64).is_err());
    }

    #[test]
    fn update_leaf() {
        let hash_4 = int_to_node(4);
        let hash_5 = int_to_node(5);
        let hash_6 = int_to_node(6);
        let hash_7 = int_to_node(7);
        let hash_45 = calculate_parent_hash(hash_4, 12u64, hash_5);
        let hash_67 = calculate_parent_hash(hash_6, 14u64, hash_7);

        let hash_0123 = int_to_node(123);

        let path_6 = vec![hash_7, hash_45, hash_0123];
        let path_5 = vec![hash_4, hash_67, hash_0123];
        let path_4 = vec![hash_5, hash_67, hash_0123];

        let index_6 = 6u64;
        let index_5 = 5u64;
        let index_4 = 4u64;
        let depth = 4u32;
        let mut set = super::MerklePathSet::new(depth).unwrap();

        set.add_path(index_6, hash_6, path_6).unwrap();
        set.add_path(index_5, hash_5, path_5).unwrap();
        set.add_path(index_4, hash_4, path_4).unwrap();

        let new_hash_6 = int_to_node(100);
        let new_hash_5 = int_to_node(55);

        set.update_leaf(index_6, new_hash_6).unwrap();
        let new_path_4 = set.get_path(depth, index_4).unwrap();
        let new_hash_67 = calculate_parent_hash(new_hash_6, 14u64, hash_7);
        assert_eq!(new_hash_67, new_path_4[1]);

        set.update_leaf(index_5, new_hash_5).unwrap();
        let new_path_4 = set.get_path(depth, index_4).unwrap();
        let new_path_6 = set.get_path(depth, index_6).unwrap();
        let new_hash_45 = calculate_parent_hash(new_hash_5, 13u64, hash_4);
        assert_eq!(new_hash_45, new_path_6[1]);
        assert_eq!(new_hash_5, new_path_4[0]);
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    const fn int_to_node(value: u64) -> Word {
        [Felt::new(value), Felt::ZERO, Felt::ZERO, Felt::ZERO]
    }
}
