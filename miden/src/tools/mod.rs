use super::cli::InputFile;
use assembly::{Assembler, AssemblyError};
use core::fmt;
use processor::{AsmOpInfo, ExecutionError};
use std::path::PathBuf;
use structopt::StructOpt;
use vm_core::{utils::collections::Vec, Operation, ProgramInputs};

// CLI
// ================================================================================================

/// Defines cli interface
#[derive(StructOpt, Debug)]
#[structopt(about = "Analyze a miden program")]
pub struct Analyze {
    /// Path to .masm assembly file
    #[structopt(short = "a", long = "assembly", parse(from_os_str))]
    assembly_file: PathBuf,
    /// Path to .inputs file
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input_file: Option<PathBuf>,
}

/// Implements CLI execution logic
impl Analyze {
    pub fn execute(&self) -> Result<(), String> {
        let program =
            std::fs::read_to_string(&self.assembly_file).expect("Could not read masm file");
        // load input data from file
        let input_data = InputFile::read(&self.input_file, &self.assembly_file)?;
        let program_info: ProgramInfo = analyze(program.as_str(), input_data.get_program_inputs())
            .expect("Could not retrieve program info");
        println!("{}", program_info);
        Ok(())
    }
}

// PROGRAM INFO
// ================================================================================================

/// Contains info of a program. Used for program analysis. Contains the following fields:
/// - total_vm_cycles: vm cycles it takes to execute the entire program
/// - total_noops: total noops executed as part of a program
/// - asm_op_stats: vector of [AsmOpStats] that contains assembly instructions and
///   the number of vm cycles it takes to execute the instruction and the number of times the
///   instruction is run as part of the given program.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ProgramInfo {
    total_vm_cycles: usize,
    total_noops: usize,
    asm_op_stats: Vec<AsmOpStats>,
}

impl ProgramInfo {
    /// Returns total vm cycles to execute a program
    pub fn total_vm_cycles(&self) -> usize {
        self.total_vm_cycles
    }

    /// Returns total noops executed as part of a program
    pub fn total_noops(&self) -> usize {
        self.total_noops
    }

    /// Returns [AsmOpStats] that contains assembly instructions and the number of vm cycles
    /// it takes to execute them and the number of times they are run as part of the given program.
    pub fn asm_op_stats(&self) -> &[AsmOpStats] {
        &self.asm_op_stats
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Increments the noop count by one
    pub fn incr_noop_count(&mut self) {
        self.total_noops += 1;
    }

    /// Sets the total vm cycles to the provided value
    pub fn set_total_vm_cycles(&mut self, total_vm_cycles: usize) {
        self.total_vm_cycles = total_vm_cycles;
    }

    /// Records a new occurrence of asmop in the sorted asmop stats vector of this program info.
    /// If the asmop is already in the list, increments its frequency by one.
    /// If the asmop is not already in the list, add it at the appropriate index to keep the
    /// list sorted alphabetically.
    pub fn record_asmop(&mut self, asmop_info: AsmOpInfo) {
        match &mut self
            .asm_op_stats
            .binary_search_by_key(&(asmop_info.op_generalized()), |asmop: &AsmOpStats| {
                asmop.op().to_string()
            }) {
            Ok(pos) => {
                if asmop_info.cycle_idx() == 1 {
                    self.asm_op_stats[*pos].incr_frequency();
                    self.asm_op_stats[*pos].add_vm_cycles(asmop_info.num_cycles());
                }
            }
            Err(pos) => {
                self.asm_op_stats.insert(
                    *pos,
                    AsmOpStats::new(
                        asmop_info.op_generalized(),
                        1,
                        asmop_info.num_cycles() as usize,
                    ),
                );
            }
        }
    }
}

impl fmt::Display for ProgramInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_vm_cycles = self.total_vm_cycles();
        let total_noops = self.total_noops();
        let asm_op_stats = self.asm_op_stats();
        writeln!(f, "Total Number of VM Cycles: {}\n", total_vm_cycles)?;
        writeln!(f, "Total Number of NOOPs executed: {}\n", total_noops)?;
        writeln!(
            f,
            "{0: <20} | {1: <20} | {2: <20} | {3: <20}",
            "AsmOp", "Frequency", "Total Cycles", "Avg Instruction Cycles"
        )?;
        for op_info in asm_op_stats {
            writeln!(
                f,
                "{0: <20} | {1: <20} | {2: <20} | {3: <20.2}",
                op_info.op(),
                op_info.frequency(),
                op_info.total_vm_cycles(),
                op_info.total_vm_cycles() as f64 / op_info.frequency() as f64
            )?;
        }
        Ok(())
    }
}

/// Returns program analysis of a given program.
pub fn analyze(program: &str, inputs: ProgramInputs) -> Result<ProgramInfo, ProgramError> {
    let assembler = Assembler::new(true);
    let program = assembler
        .compile(program)
        .map_err(ProgramError::AssemblyError)?;
    let vm_state_iterator = processor::execute_iter(&program, &inputs);
    let mut program_info = ProgramInfo::default();

    for state in vm_state_iterator {
        let vm_state = state.map_err(ProgramError::ExecutionError)?;
        if matches!(vm_state.op, Some(Operation::Noop)) {
            program_info.incr_noop_count();
        }
        if let Some(asmop_info) = vm_state.asmop {
            program_info.record_asmop(asmop_info);
        }
        program_info.set_total_vm_cycles(vm_state.clk);
    }

    Ok(program_info)
}

// PROGRAM ERROR
// ================================================================================================

/// This is used to specify the error type returned from analyze.
#[derive(Debug)]
pub enum ProgramError {
    AssemblyError(AssemblyError),
    ExecutionError(ExecutionError),
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramError::AssemblyError(e) => write!(f, "Assembly Error: {:?}", e),
            ProgramError::ExecutionError(e) => write!(f, "Execution Error: {:?}", e),
        }
    }
}

// ASMOP STATS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub struct AsmOpStats {
    op: String,
    frequency: usize,
    total_vm_cycles: usize,
}

impl AsmOpStats {
    /// Returns [AsmOpStats] instantiated with the specified assembly instruction string,
    /// number of cycles it takes to execute the assembly instruction and the number of times
    /// the assembly instruction is executed.
    pub fn new(op: String, frequency: usize, total_vm_cycles: usize) -> Self {
        Self {
            op,
            frequency,
            total_vm_cycles,
        }
    }

    /// Returns the assembly instruction corresponding to this decorator.
    pub fn op(&self) -> &String {
        &self.op
    }

    /// Returns the number of times this AsmOp is executed as part of a program.
    pub fn frequency(&self) -> usize {
        self.frequency
    }

    /// Returns the combined vm cycles all occurrences of this AsmOp take.
    pub fn total_vm_cycles(&self) -> usize {
        self.total_vm_cycles
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Increments the frequency of this AsmOp.
    pub fn incr_frequency(&mut self) {
        self.frequency += 1;
    }

    /// Increments the total vm cycles of this AsmOp by the specified number of vm cycles.
    pub fn add_vm_cycles(&mut self, num_cycles: u8) {
        self.total_vm_cycles += num_cycles as usize;
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{AsmOpStats, ProgramInfo};

    #[test]
    fn analyze_test() {
        let source =
            "proc.foo.1 pop.local.0 end begin popw.mem.1 push.17 push.1 movdn.2 exec.foo end";
        let program_inputs = super::ProgramInputs::none();
        let program_info =
            super::analyze(source, program_inputs).expect("analyze_test: Unexpected Error");
        let expected_program_info = ProgramInfo {
            total_vm_cycles: 27,
            total_noops: 1,
            asm_op_stats: vec![
                AsmOpStats::new("movdn.2".to_string(), 1, 1),
                AsmOpStats::new("pop.local".to_string(), 1, 10),
                AsmOpStats::new("popw.mem".to_string(), 1, 6),
                AsmOpStats::new("push".to_string(), 2, 3),
            ],
        };
        assert_eq!(program_info, expected_program_info);
    }

    #[test]
    fn analyze_test_execution_error() {
        let source = "begin div end";
        let stack_input = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let program_inputs = super::ProgramInputs::new(&stack_input, &[], vec![]).unwrap();
        let program_info = super::analyze(source, program_inputs);
        let expected_error = "Execution Error: DivideByZero(1)";
        assert_eq!(program_info.err().unwrap().to_string(), expected_error);
    }

    #[test]
    fn analyze_test_assembly_error() {
        let source = "proc.foo.1 pop.local.0 end popw.mem.1 push.17 exec.foo end";
        let program_inputs = super::ProgramInputs::none();
        let program_info = super::analyze(source, program_inputs);
        let expected_error = "Assembly Error: assembly error at 3: unexpected token: expected 'begin' but was 'popw.mem.1'";
        assert_eq!(program_info.err().unwrap().to_string(), expected_error);
    }
}
