mod advice;
mod asmop;
mod proc_marker;
use processor::{AsmOpInfo, ProcInfo, VmStateIterator};
use vm_core::Operation;

/// [VmStatePartial] holds the following current process state information at a specific clock cycle
/// * clk: Current clock cycle
/// * asmop: [AsmOp] decorator at the specific clock cycle
/// * op: [Operation] executed at the specific clock cycle
/// * proc_stack: stack of currently executing procedures at the specified clock cycle
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmStatePartial {
    clk: usize,
    asmop: Option<AsmOpInfo>,
    op: Option<Operation>,
    proc_stack: Vec<ProcInfo>,
}

/// This is a helper function to build a vector of [VmStatePartial] from a specified [VmStateIterator].
pub fn build_vm_state(vm_state_iterator: VmStateIterator) -> Vec<VmStatePartial> {
    let mut vm_state = Vec::new();
    for state in vm_state_iterator {
        let _proc_stack = state.as_ref().unwrap().proc_stack.clone();
        vm_state.push(VmStatePartial {
            clk: state.as_ref().unwrap().clk,
            asmop: state.as_ref().unwrap().asmop.clone(),
            op: state.as_ref().unwrap().op,
            proc_stack: state.as_ref().unwrap().proc_stack.clone(),
        });
    }
    vm_state
}
