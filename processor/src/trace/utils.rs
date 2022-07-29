use super::{Felt, FieldElement, Matrix, Vec};
use core::slice;
use vm_core::utils::uninit_vector;

// TRACE FRAGMENT
// ================================================================================================

/// TODO: add docs
pub struct TraceFragment<'a> {
    data: Vec<&'a mut [Felt]>,
}

impl<'a> TraceFragment<'a> {
    /// Creates a new TraceFragment with its data allocated to the specified capacity.
    pub fn new(capacity: usize) -> Self {
        TraceFragment {
            data: Vec::with_capacity(capacity),
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of columns in this execution trace fragment.
    pub fn width(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of rows in this execution trace fragment.
    pub fn len(&self) -> usize {
        self.data[0].len()
    }

    // DATA MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Updates a single cell in this fragment with provided value.
    #[inline(always)]
    pub fn set(&mut self, row_idx: usize, col_idx: usize, value: Felt) {
        self.data[col_idx][row_idx] = value;
    }

    /// Returns a mutable iterator to the columns of this fragment.
    pub fn columns(&mut self) -> slice::IterMut<'_, &'a mut [Felt]> {
        self.data.iter_mut()
    }

    /// Adds a new column to this fragment by pushing a mutable slice with the first `len`
    /// elements of the provided column. Returns the rest of the provided column as a separate
    /// mutable slice.
    pub fn push_column_slice(&mut self, column: &'a mut [Felt], len: usize) -> &'a mut [Felt] {
        let (column_fragment, rest) = column.split_at_mut(len);
        self.data.push(column_fragment);
        rest
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------

    #[cfg(test)]
    pub fn trace_to_fragment(trace: &'a mut [Vec<Felt>]) -> Self {
        let mut data = Vec::new();
        for column in trace.iter_mut() {
            data.push(column.as_mut_slice());
        }
        Self { data }
    }
}

// LOOKUP TABLES
// ================================================================================================

/// Defines a single row in a lookup table defined via multiset checks.
pub trait LookupTableRow {
    /// Returns a single element representing the row in the field defined by E. The value is
    /// computed using the provided random values.
    fn to_value<E: FieldElement<BaseField = Felt>>(&self, rand_values: &[E]) -> E;
}

/// Computes values as well as inverse value for all specified lookup table rows.
///
/// To compute the inverses of row values we use a modified version of batch inversion algorithm.
/// The main modification is that we don't need to check for ZERO values, because, assuming
/// random values are drawn from a large enough field, coming across a ZERO value should be
/// computationally infeasible.
pub fn build_lookup_table_row_values<E: FieldElement<BaseField = Felt>, R: LookupTableRow>(
    rows: &[R],
    rand_values: &[E],
) -> (Vec<E>, Vec<E>) {
    let mut row_values = unsafe { uninit_vector(rows.len()) };
    let mut inv_row_values = unsafe { uninit_vector(rows.len()) };

    // compute row values and compute their product
    let mut acc = E::ONE;
    for ((row, value), inv_value) in rows
        .iter()
        .zip(row_values.iter_mut())
        .zip(inv_row_values.iter_mut())
    {
        *inv_value = acc;
        *value = row.to_value(rand_values);
        debug_assert_ne!(*value, E::ZERO, "row value cannot be ZERO");

        acc *= *value;
    }

    // invert the accumulated product
    acc = acc.inv();

    // multiply the accumulated value by original values to compute inverses
    for i in (0..row_values.len()).rev() {
        inv_row_values[i] *= acc;
        acc *= row_values[i];
    }

    (row_values, inv_row_values)
}

// AUX COLUMN BUILDER
// ================================================================================================

/// Defines a builder responsible for building a single column in an auxiliary segment of the
/// execution trace.
pub trait AuxColumnBuilder<H: Copy, R: LookupTableRow> {
    // REQUIRED METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns an exhaustive list of rows which are present in the table.
    fn get_table_rows(&self) -> &[R];

    /// Returns a sequence of hints which indicate how the table was updated. Each hint consists
    /// of a clock cycle at which the update happened as well as the hint describing the update.
    fn get_table_hints(&self) -> &[(usize, H)];

    /// Returns a value by which the current value of the column should be multiplied to get the
    /// next value. It is expected that this value should never be ZERO in practice.
    fn get_multiplicand<E: FieldElement<BaseField = Felt>>(
        &self,
        hint: H,
        row_values: &[E],
        inv_row_values: &[E],
    ) -> E;

    // PROVIDED METHODS
    // --------------------------------------------------------------------------------------------

    /// Builds and returns the auxiliary trace column managed by this builder.
    fn build_aux_column<E>(&self, main_trace: &Matrix<Felt>, alphas: &[E]) -> Vec<E>
    where
        E: FieldElement<BaseField = Felt>,
    {
        // compute row values and their inverses for all rows that were added to the table
        let (row_values, inv_row_values) = self.build_row_values(main_trace, alphas);

        // allocate memory for the running product column and set its initial value
        let mut result = unsafe { uninit_vector(main_trace.num_rows()) };
        result[0] = self.init_column_value(alphas);

        // keep track of the last updated row in the running product column
        let mut result_idx = 0;

        // iterate through the list of updates and apply them one by one
        for (clk, hint) in self.get_table_hints() {
            let clk = *clk;

            // if we skipped some cycles since the last update was processed, values in the last
            // updated row should by copied over until the current cycle.
            if result_idx < clk {
                let last_value = result[result_idx];
                result[(result_idx + 1)..=clk].fill(last_value);
            }

            // move the result pointer to the next row
            result_idx = clk + 1;

            // apply the relevant updates to the column; since the multiplicand value should be
            // generated by "mixing-in" random values from a large field, the probability that we
            // get a ZERO should be negligible (i.e., it should never come up in practice).
            let multiplicand = self.get_multiplicand(*hint, &row_values, &inv_row_values);
            debug_assert_ne!(E::ZERO, multiplicand);
            result[result_idx] = result[clk] * multiplicand;
        }

        // after all updates have been processed, the table should not change; we make sure that
        // the last value in the column is equal to the expected value, and fill in all the
        // remaining column values with the last value
        let last_value = result[result_idx];
        assert_eq!(last_value, self.final_column_value(alphas));
        if result_idx < result.len() - 1 {
            result[(result_idx + 1)..].fill(last_value);
        }

        result
    }

    /// Builds and returns row values and their inverses for all rows which were added to the
    /// lookup table managed by this column builder.
    fn build_row_values<E>(&self, _main_trace: &Matrix<Felt>, alphas: &[E]) -> (Vec<E>, Vec<E>)
    where
        E: FieldElement<BaseField = Felt>,
    {
        build_lookup_table_row_values(self.get_table_rows(), alphas)
    }

    /// Returns the initial value in the auxiliary column. Default implementation of this method
    /// returns ONE.
    fn init_column_value<E: FieldElement<BaseField = Felt>>(&self, _alphas: &[E]) -> E {
        E::ONE
    }

    /// Returns the final value in the auxiliary column. Default implementation of this method
    /// returns ONE.
    fn final_column_value<E: FieldElement<BaseField = Felt>>(&self, _alphas: &[E]) -> E {
        E::ONE
    }
}
