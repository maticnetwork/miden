use crate::{Felt, Operation, Process};
use core::cmp;
use logtest::Logger;
use vm_core::{DebugOptions, ProgramInputs};

#[test]
fn test_debug() {
    // Logger can only be initialized once per module, therefore grouping all into one test.
    let mut logger = Logger::start();
    test_print_all(&mut logger);
    test_print_mem(&mut logger);
    test_print_stack(&mut logger);
    test_print_local(&mut logger);
}

fn assert_clock(process: &Process, logger: &mut Logger) {
    assert_eq!(
        logger.pop().unwrap().args(),
        format!(
            "---------------------cycle: {}---------------------",
            process.system.clk()
        )
    );
}

fn test_print_all(logger: &mut Logger) {
    let stack = (1..=16).collect::<Vec<_>>();
    let inputs = ProgramInputs::new(&stack, &[], vec![]).unwrap();
    let mut process = Process::new(inputs);
    process.memory.write(
        Felt::new(1),
        [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)],
    );
    process.memory.write(
        Felt::new(4),
        [
            Felt::new(u64::MAX - 4),
            Felt::new(u64::MAX - 3),
            Felt::new(u64::MAX - 2),
            Felt::new(u64::MAX - 1),
        ],
    );

    process
        .execute_op(&Operation::Debug(DebugOptions::All))
        .unwrap();

    assert_eq!(logger.len(), 7);
    for _ in 0..(logger.len()) {
        assert!(logger.pop().is_some());
    }
}

fn test_print_mem(logger: &mut Logger) {
    let inputs = ProgramInputs::new(&[], &[], vec![]).unwrap();
    let mut process = Process::new(inputs);
    process.memory.write(
        Felt::new(1),
        [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)],
    );
    process.memory.write(
        Felt::new(4),
        [Felt::new(5), Felt::new(6), Felt::new(7), Felt::new(8)],
    );

    // Print all memory
    process
        .execute_op(&Operation::Debug(DebugOptions::Memory(None, None)))
        .unwrap();

    assert_clock(&process, logger);
    assert_eq!(logger.len(), 3);
    let mut record = logger.pop().unwrap();
    assert_eq!(record.args(), "memory (2 of 2) ---------");
    record = logger.pop().unwrap();
    let memory_one =
        "0x00000000000001: [0x00000000000001, 0x00000000000002, 0x00000000000003, 0x00000000000004]";
    let memory_four =
        "0x00000000000004: [0x00000000000005, 0x00000000000006, 0x00000000000007, 0x00000000000008]";
    assert_eq!(record.args(), memory_one);
    record = logger.pop().unwrap();
    assert_eq!(record.args(), memory_four);

    // Print memory at address 1
    process
        .execute_op(&Operation::Debug(DebugOptions::Memory(Some(1), None)))
        .unwrap();

    assert_clock(&process, logger);
    assert_eq!(logger.len(), 2);
    record = logger.pop().unwrap();
    assert_eq!(record.args(), "memory (1 of 2) ---------");
    record = logger.pop().unwrap();
    assert_eq!(record.args(), memory_one);

    // Print memory from address 1 to 4
    process
        .execute_op(&Operation::Debug(DebugOptions::Memory(Some(1), Some(4))))
        .unwrap();

    assert_clock(&process, logger);
    assert_eq!(logger.len(), 3);
    record = logger.pop().unwrap();
    assert_eq!(record.args(), "memory (2 of 2) ---------");
    record = logger.pop().unwrap();
    assert_eq!(record.args(), memory_one);
    record = logger.pop().unwrap();
    assert_eq!(record.args(), memory_four);

    // Print unused memory
    process
        .execute_op(&Operation::Debug(DebugOptions::Memory(Some(3), None)))
        .unwrap();
    assert_clock(&process, logger);
    assert_eq!(logger.len(), 2);
    record = logger.pop().unwrap();
    assert_eq!(record.args(), "memory (0 of 2) ---------");
    record = logger.pop().unwrap();
    assert_eq!(record.args(), "0x00000000000003: <empty>");
}

fn assert_stack(size: usize, depth: usize, expected_stack: &[u64], logger: &mut Logger) {
    assert_eq!(logger.len(), 2);
    let top_size = cmp::min(size, 16);

    // build the expected debug string from the expected stack
    let mut expected = expected_stack
        .iter()
        .take(top_size)
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    for _ in 0..(size - top_size) {
        expected += ", 1";
    }
    let mut record = logger.pop().unwrap();
    assert_eq!(
        record.args(),
        format!("stack ({} of {}) ---------", size, depth)
    );
    record = logger.pop().unwrap();
    assert_eq!(record.args(), expected);
}

fn test_print_stack(logger: &mut Logger) {
    // Create a process with 4 items in the stack
    let mut stack_inputs = (1..=4).collect::<Vec<_>>();
    let mut inputs = ProgramInputs::new(&stack_inputs, &[], vec![]).unwrap();

    let mut process = Process::new(inputs);

    // Print all stack
    process
        .execute_op(&Operation::Debug(DebugOptions::Stack(None)))
        .unwrap();

    assert_clock(&process, logger);
    // values are pushed onto the stack when ProgramInputs are created, so we expect them to be on
    // the stack in reverse order from the original input order
    stack_inputs.reverse();
    assert_stack(4, 4, &stack_inputs, logger);

    stack_inputs = (1..=16).collect::<Vec<_>>();
    inputs = ProgramInputs::new(&stack_inputs, &[], vec![]).unwrap();
    process = Process::new(inputs);
    // Push two elements into the overflow table
    process.stack.shift_right(1);
    process.stack.shift_right(1);

    // Print all stack
    process
        .execute_op(&Operation::Debug(DebugOptions::Stack(None)))
        .unwrap();
    assert_clock(&process, logger);
    stack_inputs.reverse();
    assert_stack(18, 18, &stack_inputs, logger);

    // Print top 3 items in stack
    process
        .execute_op(&Operation::Debug(DebugOptions::Stack(Some(3))))
        .unwrap();
    assert_clock(&process, logger);
    assert_stack(3, 18, &stack_inputs, logger);

    // Print top 18 items in stack (includes overflow)
    process
        .execute_op(&Operation::Debug(DebugOptions::Stack(Some(18))))
        .unwrap();
    assert_clock(&process, logger);
    assert_stack(18, 18, &stack_inputs, logger);
}

fn test_print_local(logger: &mut Logger) {
    let inputs = ProgramInputs::new(&[], &[], vec![]).unwrap();
    let mut process = Process::new(inputs);
    process.system.set_fmp(Felt::new(1));
    process.memory.write(
        Felt::new(1),
        [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)],
    );

    // Print local
    process
        .execute_op(&Operation::Debug(DebugOptions::Local(None)))
        .unwrap();

    assert_clock(&process, logger);
    assert_eq!(logger.len(), 1);
    let record = logger.pop().unwrap();
    assert_eq!(
        record.args(),
        "local: [0x00000000000001, 0x00000000000002, 0x00000000000003, 0x00000000000004]"
    );
}
