use super::{Felt, FieldElement, SysTrace, Vec};

// CONSTANTS
// ================================================================================================

// Memory addresses for procedure locals should start at 2^30 and not go below.
pub const FMP_MIN: u64 = 2_u64.pow(30);
// The total number of locals available to all procedures at runtime must be smaller than 2^32.
pub const FMP_MAX: u64 = FMP_MIN + u32::MAX as u64;

// SYSTEM INFO
// ================================================================================================

/// System info container for the VM.
///
/// Currently, this keeps track of the clock cycle and free memory pointer registers.
pub struct System {
    clk: usize,
    clk_trace: Vec<Felt>,
    fmp: Felt,
    fmp_trace: Vec<Felt>,
}

impl System {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [System] struct with execution traces instantiated with the specified length.
    /// Initializes the free memory pointer `fmp` used for local memory offsets to 2^30.
    pub fn new(init_trace_capacity: usize) -> Self {
        // set the first value of the fmp trace to 2^30.
        let fmp = Felt::new(FMP_MIN);
        let mut fmp_trace = Felt::zeroed_vector(init_trace_capacity);
        fmp_trace[0] = fmp;

        Self {
            clk: 0,
            clk_trace: Felt::zeroed_vector(init_trace_capacity),
            fmp,
            fmp_trace,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the current clock cycle of a process.
    #[inline(always)]
    pub fn clk(&self) -> usize {
        self.clk
    }

    /// Returns the current value of the free memory pointer for a process.
    #[inline(always)]
    pub fn fmp(&self) -> Felt {
        self.fmp
    }

    /// Returns execution trace length for the systems columns of the process.
    ///
    /// Trace length of the system columns is equal to the number of cycles executed by the VM.
    #[inline(always)]
    pub fn trace_len(&self) -> usize {
        self.clk
    }

    /// Returns an execution trace of this system info container.
    ///
    /// If the trace is smaller than the specified `trace_len`, the columns of the trace are
    /// extended to match the specified length as follows:
    /// - the remainder of the `clk` column is filled in with increasing values of `clk`.
    /// - the remainder of the `fmp` column is filled in with the last value in the column.
    ///
    /// `num_rand_rows` indicates the number of rows at the end of the trace which will be
    /// overwritten with random values. This parameter is unused because last rows are just
    /// duplicates of the prior rows and thus can be safely overwritten.
    pub fn into_trace(mut self, trace_len: usize, num_rand_rows: usize) -> SysTrace {
        let clk = self.clk();
        // make sure that only the duplicate rows will be overwritten with random values
        assert!(
            clk + num_rand_rows <= trace_len,
            "target trace length too small"
        );

        // complete the clk column by filling in all values after the last clock cycle. The values
        // in the clk column are equal to the index of the row in the trace table.
        self.clk_trace.resize(trace_len, Felt::ZERO);
        for (i, clk) in self.clk_trace.iter_mut().enumerate().skip(clk) {
            // converting from u32 is OK here because max trace length is 2^32
            *clk = Felt::from(i as u32);
        }

        // complete the fmp column by filling in all values after the last clock cycle with the
        // value in the column at the last clock cycle.
        let last_value = self.fmp_trace[clk];
        self.fmp_trace[clk..].fill(last_value);
        self.fmp_trace.resize(trace_len, last_value);

        [self.clk_trace, self.fmp_trace]
    }

    /// Returns free memory pointer at the specified clock cycle.
    #[inline(always)]
    pub fn get_fmp_at(&self, clk: usize) -> Felt {
        self.fmp_trace[clk]
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Increments the clock cycle.
    pub fn advance_clock(&mut self) {
        self.clk += 1;
        self.clk_trace[self.clk] = Felt::new(self.clk as u64);

        self.fmp_trace[self.clk] = self.fmp;
    }

    /// Sets the value of free memory pointer for the next clock cycle.
    pub fn set_fmp(&mut self, fmp: Felt) {
        // we set only the current value of fmp here, the trace will be updated with this value
        // when the clock cycle advances.
        self.fmp = fmp;
    }

    // UTILITY METHODS
    // --------------------------------------------------------------------------------------------

    /// Makes sure there is enough memory allocated for the trace to accommodate a new row.
    ///
    /// Trace length is doubled every time it needs to be increased.
    pub fn ensure_trace_capacity(&mut self) {
        let current_capacity = self.clk_trace.len();
        if self.clk + 1 >= current_capacity {
            let new_length = current_capacity * 2;
            self.clk_trace.resize(new_length, Felt::ZERO);
            self.fmp_trace.resize(new_length, Felt::ZERO);
        }
    }
}
