use super::{
    super::trace::AuxColumnBuilder, Felt, FieldElement, OverflowTableRow, OverflowTableUpdate, Vec,
};
use winterfell::Matrix;

// AUXILIARY TRACE BUILDER
// ================================================================================================

/// Describes how to construct execution traces of stack-related auxiliary trace segment columns
/// (used in multiset checks).
pub struct AuxTraceBuilder {
    pub(super) overflow_hints: Vec<(usize, OverflowTableUpdate)>,
    pub(super) overflow_table_rows: Vec<OverflowTableRow>,
}

impl AuxTraceBuilder {
    /// Builds and returns stack auxiliary trace columns. Currently this consists of a single
    /// column p1 describing states of the stack overflow table.
    pub fn build_aux_columns<E: FieldElement<BaseField = Felt>>(
        &self,
        main_trace: &Matrix<Felt>,
        rand_elements: &[E],
    ) -> Vec<Vec<E>> {
        let p1 = self.build_aux_column(main_trace, rand_elements);
        vec![p1]
    }
}

// OVERFLOW TABLE
// ================================================================================================

impl AuxColumnBuilder<OverflowTableUpdate, OverflowTableRow> for AuxTraceBuilder {
    /// Returns a list of rows which were added to and then removed from the stack overflow table.
    ///
    /// The order of the rows in the list is the same as the order in which the rows were added to
    /// the table.
    fn get_table_rows(&self) -> &[OverflowTableRow] {
        &self.overflow_table_rows
    }

    /// Returns hints which describe how the stack overflow table was updated during program
    /// execution. Each update hint is accompanied by a clock cycle at which the update happened.
    ///
    /// Internally, each update hint also contains an index of the row into the full list of rows
    /// which was either added or removed.
    fn get_table_hints(&self) -> &[(usize, OverflowTableUpdate)] {
        &self.overflow_hints
    }

    /// Returns the value by which the running product column should be multiplied for the provided
    /// hint value.
    fn get_multiplicand<E: FieldElement<BaseField = Felt>>(
        &self,
        hint: OverflowTableUpdate,
        row_values: &[E],
        inv_row_values: &[E],
    ) -> E {
        match hint {
            OverflowTableUpdate::RowInserted(inserted_row_idx) => {
                row_values[inserted_row_idx as usize]
            }
            OverflowTableUpdate::RowRemoved(removed_row_idx) => {
                inv_row_values[removed_row_idx as usize]
            }
        }
    }
}
