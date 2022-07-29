use super::{build_vm_state, VmStatePartial};
use crate::build_debug_test;
use processor::{AsmOpInfo, ProcInfo};
use vm_core::{Felt, Operation};

#[test]
fn asmop_one_span_block_test() {
    let source = "begin push.1 push.2 add end";
    let test = build_debug_test!(source);
    let vm_state_iterator = test.execute_iter();
    let expected_vm_state = vec![
        VmStatePartial {
            clk: 0,
            asmop: None,
            op: None,
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 1,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 2,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 4,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 5,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);
}

#[test]
fn asmop_with_one_procedure() {
    let source = "proc.foo push.1 push.2 add end begin exec.foo end";
    let test = build_debug_test!(source);
    let vm_state_iterator = test.execute_iter();
    let expected_vm_state = vec![
        VmStatePartial {
            clk: 0,
            asmop: None,
            op: None,
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 1,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 2,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 4,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 5,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);
}

#[test]
fn asmop_repeat_test() {
    let source = "begin
            repeat.3
                push.1 push.2 add
            end
        end";
    let test = build_debug_test!(source);
    let vm_state_iterator = test.execute_iter();
    let expected_vm_state = vec![
        VmStatePartial {
            clk: 0,
            asmop: None,
            op: None,
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 1,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 2,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 4,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 5,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 6,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 10,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 11,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 12,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 13,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 14,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 15,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 16,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 17,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);
}

#[test]
fn asmop_conditional_execution_test() {
    let source = "begin
            eq
            if.true
                push.1 push.2 add
            else
                push.3 push.4 add
            end            
        end";

    //if branch
    let test = build_debug_test!(source, &[1, 1]);
    let vm_state_iterator = test.execute_iter();
    let expected_vm_state = vec![
        VmStatePartial {
            clk: 0,
            asmop: None,
            op: None,
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 1,
            asmop: None,
            op: Some(Operation::Join),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 2,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("eq".to_string(), 1, 1)),
            op: Some(Operation::Eq),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 4,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 5,
            asmop: None,
            op: Some(Operation::Split),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 10,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 11,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 12,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 13,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);

    //else branch
    let test = build_debug_test!(source, &[1, 0]);
    let vm_state_iterator = test.execute_iter();
    let expected_vm_state = vec![
        VmStatePartial {
            clk: 0,
            asmop: None,
            op: None,
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 1,
            asmop: None,
            op: Some(Operation::Join),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 2,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("eq".to_string(), 1, 1)),
            op: Some(Operation::Eq),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 4,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 5,
            asmop: None,
            op: Some(Operation::Split),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.3".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(3))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.4".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(4))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 10,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 11,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 12,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 13,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);
}
