use super::build_debug_test;
use processor::{AsmOpInfo, ProcInfo, VmState};
use vm_core::{utils::ToElements, Felt, FieldElement, Operation};

// EXEC ITER TESTS
// =================================================================
#[test]
fn test_exec_iter() {
    let source = "proc.foo.1 pop.local.0 end begin popw.mem.1 push.17 exec.foo end";
    let mut init_stack: Vec<u64> = Vec::new();
    (1..=16).for_each(|i| {
        init_stack.push(i);
    });
    let test = build_debug_test!(source, &init_stack);
    let traces = test.execute_iter();
    let fmp = Felt::new(2u64.pow(30));
    let next_fmp = fmp + Felt::ONE;
    let mem = vec![(1_u64, slice_to_word(&[13, 14, 15, 16]))];
    let expected_states = vec![
        VmState {
            clk: 0,
            op: None,
            asmop: None,
            proc_stack: vec![],
            stack: [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1].to_elements(),
            fmp,
            memory: Vec::new(),
        },
        VmState {
            clk: 1,
            op: Some(Operation::Span),
            asmop: None,
            proc_stack: vec![],
            stack: [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1].to_elements(),
            fmp,
            memory: Vec::new(),
        },
        VmState {
            clk: 2,
            op: Some(Operation::Push(Felt::new(1))),
            asmop: Some(AsmOpInfo::new("popw.mem.1".to_string(), 6, 1)),
            proc_stack: vec![],
            stack: [1, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2].to_elements(),
            fmp,
            memory: Vec::new(),
        },
        VmState {
            clk: 3,
            op: Some(Operation::MStoreW),
            asmop: Some(AsmOpInfo::new("popw.mem.1".to_string(), 6, 2)),
            proc_stack: vec![],
            stack: [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1].to_elements(),
            fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 4,
            op: Some(Operation::Drop),
            asmop: Some(AsmOpInfo::new("popw.mem.1".to_string(), 6, 3)),
            proc_stack: vec![],
            stack: [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0].to_elements(),
            fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 5,
            op: Some(Operation::Drop),
            asmop: Some(AsmOpInfo::new("popw.mem.1".to_string(), 6, 4)),
            proc_stack: vec![],
            stack: [14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0].to_elements(),
            fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 6,
            op: Some(Operation::Drop),
            asmop: Some(AsmOpInfo::new("popw.mem.1".to_string(), 6, 5)),
            proc_stack: vec![],
            stack: [13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0].to_elements(),
            fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 7,
            op: Some(Operation::Drop),
            asmop: Some(AsmOpInfo::new("popw.mem.1".to_string(), 6, 6)),
            proc_stack: vec![],
            stack: [12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0].to_elements(),
            fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 8,
            op: Some(Operation::Push(Felt::new(17))),
            asmop: Some(AsmOpInfo::new("push.17".to_string(), 1, 1)),
            proc_stack: vec![],
            stack: [17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0].to_elements(),
            fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 9,
            op: Some(Operation::Push(Felt::new(1))),
            asmop: None,
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [1, 17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0].to_elements(),
            fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 10,
            op: Some(Operation::FmpUpdate),
            asmop: None,
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0].to_elements(),
            fmp: next_fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 11,
            op: Some(Operation::Pad),
            asmop: Some(AsmOpInfo::new("pop.local.0".to_string(), 10, 1)),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [0, 17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0].to_elements(),
            fmp: next_fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 12,
            op: Some(Operation::Pad),
            asmop: Some(AsmOpInfo::new("pop.local.0".to_string(), 10, 2)),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [
                0, 0, 17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0,
            ]
            .to_elements(),
            fmp: next_fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 13,
            op: Some(Operation::Pad),
            asmop: Some(AsmOpInfo::new("pop.local.0".to_string(), 10, 3)),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [
                0, 0, 0, 17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 0, 0, 0, 0,
            ]
            .to_elements(),
            fmp: next_fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 14,
            op: Some(Operation::Pad),
            asmop: Some(AsmOpInfo::new("pop.local.0".to_string(), 10, 4)),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [
                0, 0, 0, 0, 17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0,
            ]
            .to_elements(),
            fmp: next_fmp,
            memory: mem.clone(),
        },
        VmState {
            clk: 15,
            op: Some(Operation::FmpAdd),
            asmop: Some(AsmOpInfo::new("pop.local.0".to_string(), 10, 5)),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [
                2u64.pow(30) + 1,
                0,
                0,
                0,
                17,
                12,
                11,
                10,
                9,
                8,
                7,
                6,
                5,
                4,
                3,
                2,
                0,
                0,
                0,
                0,
            ]
            .to_elements(),
            fmp: next_fmp,
            memory: mem,
        },
        VmState {
            clk: 16,
            op: Some(Operation::MStoreW),
            asmop: Some(AsmOpInfo::new("pop.local.0".to_string(), 10, 6)),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [0, 0, 0, 17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0].to_elements(),
            fmp: next_fmp,
            memory: vec![
                (1_u64, slice_to_word(&[13, 14, 15, 16])),
                (2u64.pow(30) + 1, slice_to_word(&[17, 0, 0, 0])),
            ],
        },
        VmState {
            clk: 17,
            op: Some(Operation::Drop),
            asmop: Some(AsmOpInfo::new("pop.local.0".to_string(), 10, 7)),
            proc_stack: vec![ProcInfo::new("foo".to_string(), 1, 9)],
            stack: [0, 0, 17, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0].to_elements(),
            fmp: next_fmp,
            memory: vec![
                (1_u64, slice_to_word(&[13, 14, 15, 16])),
                (2u64.pow(30) + 1, slice_to_word(&[17, 0, 0, 0])),
            ],
        },
    ];
    for (expected, t) in expected_states.iter().zip(traces) {
        let state = t.as_ref().unwrap();
        assert_eq!(*expected, *state);
    }
}

// HELPER FUNCTIONS
// =================================================================
fn slice_to_word(values: &[i32]) -> [Felt; 4] {
    [
        Felt::new(values[0] as u64),
        Felt::new(values[1] as u64),
        Felt::new(values[2] as u64),
        Felt::new(values[3] as u64),
    ]
}
