use crate::utils::string::String;

// ASSEMBLY OP
// ================================================================================================

/// Contains information corresponsing to an assembly instruction (only applicable in debug mode)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssemblyOp {
    op: String,
    num_cycles: u8,
}

impl AssemblyOp {
    /// Returns [AssemblyOp] instantiated with the specified assembly instruction string and number
    /// of cycles it takes to execute the assembly instruction.
    pub fn new(op: String, num_cycles: u8) -> Self {
        Self { op, num_cycles }
    }

    /// Returns the assembly instruction corresponding to this decorator.
    pub fn op(&self) -> &String {
        &self.op
    }

    /// Returns the number of VM cycles taken to execute the assembly instruction of this decorator.
    pub fn num_cycles(&self) -> u8 {
        self.num_cycles
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Change cycles corresponding to an AsmOp decorator to the specified number of cycles.
    pub fn set_num_cycles(&mut self, num_cycles: u8) {
        self.num_cycles = num_cycles;
    }
}
