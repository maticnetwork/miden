use super::{ChipletsLookup, ChipletsLookupRow, Felt, FieldElement};
use crate::{
    trace::{build_lookup_table_row_values, AuxColumnBuilder, LookupTableRow},
    Vec,
};
use winterfell::Matrix;

// AUXILIARY TRACE BUILDER
// ================================================================================================

/// Describes how to construct execution traces of auxiliary trace columns that depend on multiple
/// chiplets in the Chiplets module (used in multiset checks).
pub struct AuxTraceBuilder {
    pub(super) lookup_hints: Vec<(usize, ChipletsLookup)>,
    pub(super) request_rows: Vec<ChipletsLookupRow>,
    pub(super) response_rows: Vec<ChipletsLookupRow>,
}

impl AuxTraceBuilder {
    // COLUMN TRACE CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Builds and returns the Chiplets's auxiliary trace columns. Currently this consists of
    /// a single bus column `b_aux` describing chiplet lookups requested by the stack and
    /// provided by chiplets in the Chiplets module.
    pub fn build_aux_columns<E: FieldElement<BaseField = Felt>>(
        &self,
        main_trace: &Matrix<Felt>,
        rand_elements: &[E],
    ) -> Vec<Vec<E>> {
        let b_aux = self.build_aux_column(main_trace, rand_elements);
        vec![b_aux]
    }
}

// CHIPLETS LOOKUPS
// ================================================================================================

impl AuxColumnBuilder<ChipletsLookup, ChipletsLookupRow> for AuxTraceBuilder {
    /// This method is required, but because it is only called inside `build_row_values` which is
    /// overridden below, it is not used here and should not be called.
    fn get_table_rows(&self) -> &[ChipletsLookupRow] {
        unimplemented!()
    }

    /// Returns hints which describe the [Chiplets] lookup requests and responses during program
    /// execution. Each update hint is accompanied by a clock cycle at which the update happened.
    ///
    /// Internally, each update hint also contains an index of the row into the full list of request
    /// rows or response rows, depending on whether it is a request, a response, or both (in which
    /// case it contains 2 indices).
    fn get_table_hints(&self) -> &[(usize, ChipletsLookup)] {
        &self.lookup_hints
    }

    /// Returns the value by which the running product column should be multiplied for the provided
    /// hint value.
    fn get_multiplicand<E: FieldElement<BaseField = Felt>>(
        &self,
        hint: ChipletsLookup,
        row_values: &[E],
        inv_row_values: &[E],
    ) -> E {
        match hint {
            ChipletsLookup::Request(request_row) => inv_row_values[request_row],
            ChipletsLookup::Response(response_row) => row_values[response_row],
            ChipletsLookup::RequestAndResponse((request_row, response_row)) => {
                inv_row_values[request_row] * row_values[response_row]
            }
        }
    }

    /// Build the row values and inverse values used to build the auxiliary column.
    ///
    /// The row values to be included come from the responses and the inverse values come from
    /// requests. Since responses are grouped by chiplet, the operation order for the requests and
    /// responses will be permutations of each other rather than sharing the same order. Therefore,
    /// the `row_values` and `inv_row_values` must be built separately.
    fn build_row_values<E>(&self, _main_trace: &Matrix<Felt>, alphas: &[E]) -> (Vec<E>, Vec<E>)
    where
        E: FieldElement<BaseField = Felt>,
    {
        // get the row values from the resonse rows
        let row_values = self
            .response_rows
            .iter()
            .map(|response| response.to_value(alphas))
            .collect();
        // get the inverse values from the request rows
        let (_, inv_row_values) = build_lookup_table_row_values(&self.request_rows, alphas);

        (row_values, inv_row_values)
    }
}
