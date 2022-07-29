use super::{Felt, FieldElement, StarkField, Vec, Word};
use crate::trace::{AuxColumnBuilder, LookupTableRow};
use winterfell::Matrix;

// AUXILIARY TRACE BUILDER
// ================================================================================================

/// Contains all relevant information and describes how to construct execution trace of hasher-
/// related auxiliary trace columns (used in multiset checks).
///
/// Currently, this manages construction of a single column representing the state of the sibling
/// table (used in Merkle root update computation).
#[derive(Debug, Clone, Default)]
pub struct AuxTraceBuilder {
    pub(super) sibling_hints: Vec<(usize, SiblingTableUpdate)>,
    pub(super) sibling_rows: Vec<SiblingTableRow>,
}

impl AuxTraceBuilder {
    // COLUMN TRACE CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Builds and returns hasher auxiliary trace columns. Currently this consists of a single
    /// column p1 describing states of the hasher sibling table (used for Merkle root update
    /// computation).
    pub fn build_aux_columns<E: FieldElement<BaseField = Felt>>(
        &self,
        main_trace: &Matrix<Felt>,
        rand_elements: &[E],
    ) -> Vec<Vec<E>> {
        let p1 = self.build_aux_column(main_trace, rand_elements);
        vec![p1]
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Specifies that an entry for the provided sibling was added to the sibling table at the
    /// specified step.
    pub fn sibling_added(&mut self, step: usize, index: Felt, sibling: Word) {
        let row_index = self.sibling_rows.len();
        let update = SiblingTableUpdate::SiblingAdded(row_index as u32);
        self.sibling_hints.push((step, update));
        self.sibling_rows.push(SiblingTableRow::new(index, sibling));
    }

    /// Specifies that an entry for a sibling was removed from the sibling table. The entry is
    /// defined by the provided offset. For example, if row_offset = 2, the second from the last
    /// entry was removed from the table.
    pub fn sibling_removed(&mut self, step: usize, row_offset: usize) {
        let row_index = self.sibling_rows.len() - row_offset - 1;
        let update = SiblingTableUpdate::SiblingRemoved(row_index as u32);
        self.sibling_hints.push((step, update));
    }
}

impl AuxColumnBuilder<SiblingTableUpdate, SiblingTableRow> for AuxTraceBuilder {
    /// Returns a list of rows which were added to and then removed from the sibling table.
    ///
    /// The order of the rows in the list is the same as the order in which the rows were added to
    /// the table.
    fn get_table_rows(&self) -> &[SiblingTableRow] {
        &self.sibling_rows
    }

    /// Returns hints which describe how the sibling table was updated during program execution.
    /// Each update hint is accompanied by a clock cycle at which the update happened.
    ///
    /// Internally, each update hint also contains an index of the row into the full list of rows
    /// which was either added or removed.
    fn get_table_hints(&self) -> &[(usize, SiblingTableUpdate)] {
        &self.sibling_hints
    }

    /// Returns the value by which the running product column should be multiplied for the provided
    /// hint value.
    fn get_multiplicand<E: FieldElement<BaseField = Felt>>(
        &self,
        hint: SiblingTableUpdate,
        row_values: &[E],
        inv_row_values: &[E],
    ) -> E {
        match hint {
            SiblingTableUpdate::SiblingAdded(inserted_row_idx) => {
                row_values[inserted_row_idx as usize]
            }
            SiblingTableUpdate::SiblingRemoved(removed_row_idx) => {
                inv_row_values[removed_row_idx as usize]
            }
        }
    }
}

// SIBLING TABLE
// ================================================================================================

/// Describes updates to the sibling table. The internal u32 values are indexes of added/removed
/// rows in a list of rows sorted chronologically (i.e., from first added row to last).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SiblingTableUpdate {
    SiblingAdded(u32),
    SiblingRemoved(u32),
}

/// Describes a single entry in the sibling table which consists of a tuple `(index, node)` where
/// index is the index of the node at its depth. For example, assume a leaf has index n. For the
/// leaf's parent the index will be n << 1. For the parent of the parent, the index will be
/// n << 2 etc.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SiblingTableRow {
    index: Felt,
    sibling: Word,
}

impl SiblingTableRow {
    pub fn new(index: Felt, sibling: Word) -> Self {
        Self { index, sibling }
    }
}

impl LookupTableRow for SiblingTableRow {
    /// Reduces this row to a single field element in the field specified by E. This requires
    /// at least 6 alpha values.
    fn to_value<E: FieldElement<BaseField = Felt>>(&self, alphas: &[E]) -> E {
        // when the least significant bit of the index is 0, the sibling will be in the 3rd word
        // of the hasher state, and when the least significant bit is 1, it will be in the 2nd
        // word. we compute the value in this way to make constraint evaluation a bit easier since
        // we need to compute the 2nd and the 3rd word values for other purposes as well.
        let lsb = self.index.as_int() & 1;
        if lsb == 0 {
            alphas[0]
                + alphas[3].mul_base(self.index)
                + alphas[12].mul_base(self.sibling[0])
                + alphas[13].mul_base(self.sibling[1])
                + alphas[14].mul_base(self.sibling[2])
                + alphas[15].mul_base(self.sibling[3])
        } else {
            alphas[0]
                + alphas[3].mul_base(self.index)
                + alphas[8].mul_base(self.sibling[0])
                + alphas[9].mul_base(self.sibling[1])
                + alphas[10].mul_base(self.sibling[2])
                + alphas[11].mul_base(self.sibling[3])
        }
    }
}
