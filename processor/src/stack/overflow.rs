use super::{
    super::trace::LookupTableRow, AuxTraceBuilder, BTreeMap, Felt, FieldElement, Vec, ZERO,
};

// OVERFLOW TABLE
// ================================================================================================

/// Stores stack values beyond the top 16 elements as well as the data which can be used in
/// construction of the auxiliary trace column describing the state of the overflow table at every
/// VM cycle.
///
/// When `trace_enabled` is set to true, we also record all changes to the table so that we can
/// reconstruct the overflow table at any clock cycle. This can be used for debugging purposes.
pub struct OverflowTable {
    /// A list of all rows that were added to and then removed from the overflow table.
    all_rows: Vec<OverflowTableRow>,
    /// A list of indexes into the `all_rows` vector with describes the rows currently in the
    /// overflow table.
    active_rows: Vec<usize>,
    /// A list of updates made to the overflow table during program execution. For each update we
    /// also track the cycle at which the update happened.
    update_trace: Vec<(usize, OverflowTableUpdate)>,
    /// A map which records the full state of the overflow table at every cycle during which an
    /// update happened. This map is populated only when `trace_enabled` = true.
    trace: BTreeMap<usize, Vec<Felt>>,
    /// A flag which specifies whether we should record the full state of the overflow table
    /// whenever an update happens. This is set to true only when executing programs for debug
    /// purposes.
    trace_enabled: bool,
}

impl OverflowTable {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [OverflowTable]. The returned table is empty.
    pub fn new(enable_trace: bool) -> Self {
        Self {
            all_rows: Vec::new(),
            active_rows: Vec::new(),
            update_trace: Vec::new(),
            trace: BTreeMap::new(),
            trace_enabled: enable_trace,
        }
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Pushes the specified value into the overflow table.
    pub fn push(&mut self, value: Felt, clk: usize) {
        // get the clock cycle of the row currently at the top of the overflow table. if the
        // overflow table is empty, this is set to ZERO.
        let prev = self
            .active_rows
            .last()
            .map_or(ZERO, |&last_row_idx| self.all_rows[last_row_idx].clk);

        // create and record the new row, and also put it at the top of the overflow table
        let row_idx = self.all_rows.len();
        self.all_rows.push(OverflowTableRow::new(clk, value, prev));
        self.active_rows.push(row_idx);

        // mark this clock cycle as the cycle at which a new row was inserted into the table
        self.update_trace
            .push((clk, OverflowTableUpdate::RowInserted(row_idx as u32)));

        if self.trace_enabled {
            // insert a copy of the current table state into the trace
            self.trace.insert(clk, self.get_values());
        }
    }

    /// Removes the last value from the overflow table and returns it together with the clock
    /// cycle of the next value in the table.
    ///
    /// If after the top value is removed the table is empty, the returned clock cycle is ZERO.
    pub fn pop(&mut self, clk: usize) -> (Felt, Felt) {
        // remove the top entry from the table and determine which table row corresponds to it
        let last_row_idx = self.active_rows.pop().expect("overflow table is empty");
        let last_row = &self.all_rows[last_row_idx];

        // mark this clock cycle as the clock cycle at which a row was removed from the table
        self.update_trace
            .push((clk, OverflowTableUpdate::RowRemoved(last_row_idx as u32)));

        if self.trace_enabled {
            // insert a copy of the current table state into the trace
            self.trace.insert(clk, self.get_values());
        }

        // return the removed value as well as the clock cycle of the value currently at the
        // top of the table
        (last_row.val, last_row.prev)
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Appends the top n values from the overflow table to the end of the provided vector.
    pub fn append_into(&self, target: &mut Vec<Felt>, n: usize) {
        for &idx in self.active_rows.iter().rev().take(n) {
            target.push(self.all_rows[idx].val);
        }
    }

    /// Appends the state of the overflow table at the specified clock cycle to the provided vector.
    pub fn append_state_into(&self, target: &mut Vec<Felt>, clk: usize) {
        if let Some(x) = self.trace.range(0..=clk).last() {
            for item in x.1.iter().rev() {
                target.push(*item);
            }
        }
    }

    /// Returns a vector consisting of just the value portion of each table row.
    fn get_values(&self) -> Vec<Felt> {
        self.active_rows
            .iter()
            .map(|&idx| self.all_rows[idx].val)
            .collect()
    }

    // AUX TRACE BUILDER GENERATION
    // --------------------------------------------------------------------------------------------

    /// Converts this [OverflowTable] into an auxiliary trace builder which can be used to construct
    /// the auxiliary trace column describing the state of the overflow table at every cycle.
    pub fn into_aux_builder(self) -> AuxTraceBuilder {
        AuxTraceBuilder {
            overflow_hints: self.update_trace,
            overflow_table_rows: self.all_rows,
        }
    }
}

// OVERFLOW TABLE ROW
// ================================================================================================

/// A single row in the stack overflow table. Each row contains the following values:
/// - The value of the stack item pushed into the overflow table.
/// - The clock cycle at which the stack item was pushed into the overflow table.
/// - The clock cycle of the value which was at the top of the overflow table when this value
///   was pushed onto it.
pub struct OverflowTableRow {
    val: Felt,
    clk: Felt,
    prev: Felt,
}

impl OverflowTableRow {
    pub fn new(clk: usize, val: Felt, prev: Felt) -> Self {
        Self {
            val,
            clk: Felt::new(clk as u64),
            prev,
        }
    }
}

impl LookupTableRow for OverflowTableRow {
    /// Reduces this row to a single field element in the field specified by E. This requires
    /// at least 4 alpha values.
    fn to_value<E: FieldElement<BaseField = Felt>>(&self, alphas: &[E]) -> E {
        alphas[0]
            + alphas[1].mul_base(self.clk)
            + alphas[2].mul_base(self.val)
            + alphas[3].mul_base(self.prev)
    }
}

// OVERFLOW TABLE UPDATES
// ================================================================================================

/// Describes an update to the stack overflow table. There could be two types of updates:
/// - A single row can be added to the table. This happens during a right shift.
/// - A single row can be removed from the table. This happens during a left shift.
///
/// For each update we also record the index of the row that was added/removed from the table.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OverflowTableUpdate {
    RowInserted(u32),
    RowRemoved(u32),
}
