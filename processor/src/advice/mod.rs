use super::{ExecutionError, Felt, ProgramInputs, Word};
use vm_core::{
    utils::{
        collections::{BTreeMap, Vec},
        IntoBytes,
    },
    AdviceSet, StarkField,
};

// ADVICE PROVIDER
// ================================================================================================

/// An advice provider supplies non-deterministic inputs to the processor during program execution.
///
/// The provider manages two types of inputs:
/// 1. An advice tape, from which the program can read elements sequentially. Once read, the
///    element is removed from the tape.
/// 2. Advice sets, which can be identified by their roots. Advice sets are views into Merkle
///    trees and can be used to provide Merkle paths.
///
/// An advice provider can be instantiated from [ProgramInputs].
pub struct AdviceProvider {
    step: usize,
    tape: Vec<Felt>,
    sets: BTreeMap<[u8; 32], AdviceSet>,
}

impl AdviceProvider {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new advice provider instantiated from the specified program inputs.
    pub fn new(inputs: ProgramInputs) -> Self {
        let (_, mut advice_tape, advice_sets) = inputs.into_parts();

        // reverse the advice tape so that we can pop elements off the end
        advice_tape.reverse();

        Self {
            step: 0,
            tape: advice_tape,
            sets: advice_sets,
        }
    }

    // ADVICE TAPE
    // --------------------------------------------------------------------------------------------

    /// Removes the next element from the advice tape and returns it.
    ///
    /// # Errors
    /// Returns an error if the advice tape is empty.
    pub fn read_tape(&mut self) -> Result<Felt, ExecutionError> {
        self.tape
            .pop()
            .ok_or(ExecutionError::EmptyAdviceTape(self.step))
    }

    /// Writes the provided value at the head of the advice tape.
    pub fn write_tape(&mut self, value: Felt) {
        self.tape.push(value);
    }

    // ADVISE SETS
    // --------------------------------------------------------------------------------------------

    /// Returns true if the advice set with the specified root is present in this advice provider.
    #[cfg(test)]
    pub fn has_advice_set(&self, root: Word) -> bool {
        self.sets.contains_key(&root.into_bytes())
    }

    /// Returns a node at the specified index in a Merkle tree with the specified root.
    ///
    /// # Errors
    /// Returns an error if:
    /// - A Merkle tree for the specified root cannot be found in this advice provider.
    /// - The specified depth is either zero or greater than the depth of the Merkle tree
    ///   identified by the specified root.
    /// - Value of the node at the specified depth and index is not known to this advice provider.
    #[allow(dead_code)]
    pub fn get_tree_node(
        &mut self,
        root: Word,
        depth: Felt,
        index: Felt,
    ) -> Result<Word, ExecutionError> {
        // look up the advice set and return an error if none is found
        let advice_set = self
            .sets
            .get(&root.into_bytes())
            .ok_or_else(|| ExecutionError::AdviceSetNotFound(root.into_bytes()))?;

        // get the tree node from the advice set based on depth and index
        let node = advice_set
            .get_node(depth.as_int() as u32, index.as_int())
            .map_err(ExecutionError::AdviceSetLookupFailed)?;

        Ok(node)
    }

    /// Returns a path to a node at the specified index in a Merkle tree with the specified root.
    ///
    /// # Errors
    /// Returns an error if:
    /// - A Merkle tree for the specified root cannot be found in this advice provider.
    /// - The specified depth is either zero or greater than the depth of the Merkle tree
    ///   identified by the specified root.
    /// - Path to the node at the specified depth and index is not known to this advice provider.
    pub fn get_merkle_path(
        &mut self,
        root: Word,
        depth: Felt,
        index: Felt,
    ) -> Result<Vec<Word>, ExecutionError> {
        // look up the advice set and return an error if none is found
        let advice_set = self
            .sets
            .get(&root.into_bytes())
            .ok_or_else(|| ExecutionError::AdviceSetNotFound(root.into_bytes()))?;

        // get the Merkle path from the advice set based on depth and index
        let path = advice_set
            .get_path(depth.as_int() as u32, index.as_int())
            .map_err(ExecutionError::AdviceSetLookupFailed)?;

        Ok(path)
    }

    /// Updates a leaf at the specified index in the advice set with the specified root with the
    /// provided value and returns a Merkle path to this leaf.
    ///
    /// If `update_in_copy` is set to true, the update is made in the copy of the specified advice
    /// set, and the old advice set is retained in this provider. Otherwise, the old advice set is
    /// removed from this provider.
    ///
    /// # Errors
    /// Returns an error if:
    /// - A Merkle tree for the specified root cannot be found in this advice provider.
    /// - The specified depth is either zero or greater than the depth of the Merkle tree
    ///   identified by the specified root.
    /// - Path to the leaf at the specified index in the specified Merkle tree is not known to this
    ///   advice provider.
    pub fn update_merkle_leaf(
        &mut self,
        root: Word,
        index: Felt,
        leaf_value: Word,
        update_in_copy: bool,
    ) -> Result<Vec<Word>, ExecutionError> {
        // look up the advice set and return error if none is found. if we are updating a copy,
        // clone the advice set; otherwise remove it from the map because the root will change,
        // and we'll re-insert the set later under a different root.
        let mut advice_set = if update_in_copy {
            // look up the advice set and return an error if none is found
            self.sets
                .get(&root.into_bytes())
                .ok_or_else(|| ExecutionError::AdviceSetNotFound(root.into_bytes()))?
                .clone()
        } else {
            self.sets
                .remove(&root.into_bytes())
                .ok_or_else(|| ExecutionError::AdviceSetNotFound(root.into_bytes()))?
        };

        // get the Merkle path from the advice set for the leaf at the specified index
        let path = advice_set
            .get_path(advice_set.depth(), index.as_int())
            .map_err(ExecutionError::AdviceSetLookupFailed)?;

        // update the advice set and re-insert it into the map
        advice_set
            .update_leaf(index.as_int(), leaf_value)
            .map_err(ExecutionError::AdviceSetLookupFailed)?;
        self.sets.insert(advice_set.root().into_bytes(), advice_set);

        Ok(path)
    }

    // CONTEXT MANAGEMENT
    // --------------------------------------------------------------------------------------------

    /// Increments the clock cycle.
    pub fn advance_clock(&mut self) {
        self.step += 1;
    }
}
