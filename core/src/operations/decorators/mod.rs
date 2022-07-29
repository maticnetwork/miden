mod advice;
mod assembly_op;
mod proc_marker;
use crate::utils::collections::Vec;
pub use advice::AdviceInjector;
pub use assembly_op::AssemblyOp;
use core::fmt;
pub use proc_marker::ProcMarker;

// DECORATORS
// ================================================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decorator {
    /// Injects zero or more values at the head of the advice tape as specified by the injector.
    /// This operation affects only the advice tape, but has no effect on other VM components
    /// (e.g., stack, memory), and does not advance VM clock.
    Advice(AdviceInjector),
    /// Adds information about the assembly instruction at a particular index
    /// (only applicable in debug mode).
    AsmOp(AssemblyOp),
    /// Adds procedure marker decorator to a program. Used to mark the beginning and end of a
    /// procedure being executed (only applicable in debug mode).
    ProcMarker(ProcMarker),
}

impl fmt::Display for Decorator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Advice(injector) => write!(f, "advice({})", injector),
            Self::AsmOp(assembly_op) => {
                write!(
                    f,
                    "asmOp({}, {})",
                    assembly_op.op(),
                    assembly_op.num_cycles()
                )
            }
            Self::ProcMarker(marker) => match marker {
                ProcMarker::ProcStarted(label, num_locals) => {
                    if *num_locals == 0 {
                        write!(f, "procStarted({})", label)
                    } else {
                        write!(f, "procStarted({}.{})", label, num_locals)
                    }
                }
                ProcMarker::ProcEnded => write!(f, "procEnded"),
            },
        }
    }
}

/// Vector consisting of a tuple of operation index (within a span block) and decorator at that index
pub type DecoratorList = Vec<(usize, Decorator)>;

/// Iterator used to iterate through the decorator list of a span block
/// while executing operation batches of a span block.
pub struct DecoratorIterator<'a> {
    decorators: &'a DecoratorList,
    idx: usize,
}

impl<'a> DecoratorIterator<'a> {
    /// Returns a new instance of decorator iterator instantiated with the provided decorator list.
    pub fn new(decorators: &'a DecoratorList) -> Self {
        Self { decorators, idx: 0 }
    }

    /// Returns the next decorator at the specified position.
    /// - Returns the decorator if a decorator at the specified position exists and increments the internal pointer.
    /// - Returns None if no decorator is to be executed at the specified position.
    #[inline(always)]
    pub fn next(&mut self, pos: usize) -> Option<&Decorator> {
        if self.idx < self.decorators.len() && self.decorators[self.idx].0 == pos {
            self.idx += 1;
            Some(&self.decorators[self.idx - 1].1)
        } else {
            None
        }
    }
}
