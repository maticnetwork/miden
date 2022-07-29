use super::{build_vm_state, VmStatePartial};
use crate::build_debug_test;
use processor::{AsmOpInfo, ProcInfo};
use vm_core::{utils::string::ToString, Felt, Operation};

#[test]
fn proc_with_one_span() {
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
fn proc_inside_repeat() {
    let source = "
        proc.foo push.1 push.2 add end
        begin
            repeat.3
                exec.foo
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
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 6)],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 6)],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 6)],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 6)],
        },
        VmStatePartial {
            clk: 10,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 10)],
        },
        VmStatePartial {
            clk: 11,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 10)],
        },
        VmStatePartial {
            clk: 12,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 10)],
        },
        VmStatePartial {
            clk: 13,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 10)],
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
fn proc_with_repeat() {
    let source = "
        proc.foo repeat.3 push.1 push.2 add end end
        begin
            exec.foo
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
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 10,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 11,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 12,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 13,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 2)],
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
fn proc_conditional_execution() {
    let source = "
        proc.foo
            if.true
                push.1 push.2 add
            else
                push.3 push.4 add
            end
        end
        begin
            eq
            exec.foo            
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
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 10,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 11,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 12,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
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
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.3".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(3))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.4".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(4))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 10,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 11,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 12,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
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

#[test]
fn proc_split_inside_join() {
    let source = "
        proc.foo
            eq
            if.true
                push.1 push.2 add
            else
                push.3 push.4 add
            end
        end
        begin
            exec.foo            
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
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 2,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("eq".to_string(), 1, 1)),
            op: Some(Operation::Eq),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 4,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 5,
            asmop: None,
            op: Some(Operation::Split),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 1)),
            op: Some(Operation::Pad),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.1".to_string(), 2, 2)),
            op: Some(Operation::Incr),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 10,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 11,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 12,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
        VmStatePartial {
            clk: 13,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 1)],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);
}

#[test]
fn proc_nested_procedures() {
    let source = "
        proc.foo push.3 push.7 mul end
        proc.bar push.5 exec.foo add end
        begin push.2 push.4 add exec.foo push.11 exec.bar sub end";

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
            asmop: Some(AsmOpInfo::new("push.2".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(2))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("push.4".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(4))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 4,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 5,
            asmop: Some(AsmOpInfo::new("push.3".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(3))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 6,
            asmop: Some(AsmOpInfo::new("push.7".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(7))),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 7,
            asmop: Some(AsmOpInfo::new("mul".to_string(), 1, 1)),
            op: Some(Operation::Mul),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 0, 5)],
        },
        VmStatePartial {
            clk: 8,
            asmop: Some(AsmOpInfo::new("push.11".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(11))),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 9,
            asmop: Some(AsmOpInfo::new("push.5".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(5))),
            proc_stack: vec![ProcInfo::new("bar".to_string(), 0, 9)],
        },
        VmStatePartial {
            clk: 10,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![ProcInfo::new("bar".to_string(), 0, 9)],
        },
        VmStatePartial {
            clk: 11,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![ProcInfo::new("bar".to_string(), 0, 9)],
        },
        VmStatePartial {
            clk: 12,
            asmop: None,
            op: Some(Operation::Respan),
            proc_stack: vec![ProcInfo::new("bar".to_string(), 0, 9)],
        },
        VmStatePartial {
            clk: 13,
            asmop: Some(AsmOpInfo::new("push.3".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(3))),
            proc_stack: vec![
                ProcInfo::new("bar".to_string(), 0, 9),
                ProcInfo::new("foo".to_string(), 0, 13),
            ],
        },
        VmStatePartial {
            clk: 14,
            asmop: Some(AsmOpInfo::new("push.7".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(7))),
            proc_stack: vec![
                ProcInfo::new("bar".to_string(), 0, 9),
                ProcInfo::new("foo".to_string(), 0, 13),
            ],
        },
        VmStatePartial {
            clk: 15,
            asmop: Some(AsmOpInfo::new("mul".to_string(), 1, 1)),
            op: Some(Operation::Mul),
            proc_stack: vec![
                ProcInfo::new("bar".to_string(), 0, 9),
                ProcInfo::new("foo".to_string(), 0, 13),
            ],
        },
        VmStatePartial {
            clk: 16,
            asmop: Some(AsmOpInfo::new("add".to_string(), 1, 1)),
            op: Some(Operation::Add),
            proc_stack: vec![ProcInfo::new("bar".to_string(), 0, 9)],
        },
        VmStatePartial {
            clk: 17,
            asmop: Some(AsmOpInfo::new("sub".to_string(), 2, 1)),
            op: Some(Operation::Neg),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 18,
            asmop: Some(AsmOpInfo::new("sub".to_string(), 2, 2)),
            op: Some(Operation::Add),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 19,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 20,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);
}

#[test]
fn multiple_procs_end_at_same_clk() {
    let source = "
        proc.foo push.3 end
        proc.bar push.5 exec.foo end
        begin exec.bar end";

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
            asmop: Some(AsmOpInfo::new("push.5".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(5))),
            proc_stack: vec![ProcInfo::new("bar".to_string(), 0, 2)],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("push.3".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(3))),
            proc_stack: vec![
                ProcInfo::new("bar".to_string(), 0, 2),
                ProcInfo::new("foo".to_string(), 0, 3),
            ],
        },
        VmStatePartial {
            clk: 4,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![],
        },
        VmStatePartial {
            clk: 5,
            asmop: None,
            op: Some(Operation::Noop),
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
fn multiple_split_start_at_same_cycle() {
    let source = "
        proc.foo if.true push.3 else push.4 end end
        proc.bar exec.foo end
        begin exec.bar end";

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
            op: Some(Operation::Split),
            proc_stack: vec![
                ProcInfo::new("foo".to_string(), 0, 1),
                ProcInfo::new("bar".to_string(), 0, 1),
            ],
        },
        VmStatePartial {
            clk: 2,
            asmop: None,
            op: Some(Operation::Span),
            proc_stack: vec![
                ProcInfo::new("foo".to_string(), 0, 1),
                ProcInfo::new("bar".to_string(), 0, 1),
            ],
        },
        VmStatePartial {
            clk: 3,
            asmop: Some(AsmOpInfo::new("push.4".to_string(), 1, 1)),
            op: Some(Operation::Push(Felt::new(4))),
            proc_stack: vec![
                ProcInfo::new("foo".to_string(), 0, 1),
                ProcInfo::new("bar".to_string(), 0, 1),
            ],
        },
        VmStatePartial {
            clk: 4,
            asmop: None,
            op: Some(Operation::Noop),
            proc_stack: vec![
                ProcInfo::new("foo".to_string(), 0, 1),
                ProcInfo::new("bar".to_string(), 0, 1),
            ],
        },
        VmStatePartial {
            clk: 5,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![
                ProcInfo::new("foo".to_string(), 0, 1),
                ProcInfo::new("bar".to_string(), 0, 1),
            ],
        },
        VmStatePartial {
            clk: 6,
            asmop: None,
            op: Some(Operation::End),
            proc_stack: vec![
                ProcInfo::new("foo".to_string(), 0, 1),
                ProcInfo::new("bar".to_string(), 0, 1),
            ],
        },
    ];
    let vm_state = build_vm_state(vm_state_iterator);
    assert_eq!(expected_vm_state, vm_state);
}
