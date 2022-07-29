use super::{
    super::{ChipletsLookup, ChipletsLookupRow},
    ChipletsBus, Felt, FieldElement, Memory, MemoryLookup, StarkField, TraceFragment, ONE, ZERO,
};
use vm_core::MEMORY_TRACE_WIDTH;

#[test]
fn mem_init() {
    let mem = Memory::new();
    assert_eq!(0, mem.size());
    assert_eq!(0, mem.trace_len());
}

#[test]
fn mem_read() {
    let mut mem = Memory::new();

    // read a value from address 0; clk = 1
    mem.advance_clock();
    let addr0 = Felt::new(0);
    let value = mem.read(addr0);
    assert_eq!([ZERO; 4], value);
    assert_eq!(1, mem.size());
    assert_eq!(1, mem.trace_len());

    // read a value from address 3; clk = 2
    mem.advance_clock();
    let addr3 = Felt::new(3);
    let value = mem.read(addr3);
    assert_eq!([ZERO; 4], value);
    assert_eq!(2, mem.size());
    assert_eq!(2, mem.trace_len());

    // read a value from address 0 again; clk = 3
    mem.advance_clock();
    let value = mem.read(addr0);
    assert_eq!([ZERO; 4], value);
    assert_eq!(2, mem.size());
    assert_eq!(3, mem.trace_len());

    // read a value from address 2; clk = 4
    mem.advance_clock();
    let addr2 = Felt::new(2);
    let value = mem.read(addr2);
    assert_eq!([ZERO; 4], value);
    assert_eq!(3, mem.size());
    assert_eq!(4, mem.trace_len());

    // check generated trace and memory data provided to the ChipletsBus; rows should be sorted by
    // address and then clock cycle
    let (trace, chiplets_bus) = build_trace(mem, 4);

    // address 0
    let mut prev_row = [ZERO; MEMORY_TRACE_WIDTH];
    let memory_access = MemoryLookup::new(addr0, 1, [ZERO; 4], [ZERO; 4]);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 0, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr0, 3, [ZERO; 4], [ZERO; 4]);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 1, &memory_access, prev_row);

    // address 2
    let memory_access = MemoryLookup::new(addr2, 4, [ZERO; 4], [ZERO; 4]);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 2, &memory_access, prev_row);

    // address 3
    let memory_access = MemoryLookup::new(addr3, 2, [ZERO; 4], [ZERO; 4]);
    verify_memory_access(&trace, &chiplets_bus, 3, &memory_access, prev_row);
}

#[test]
fn mem_write() {
    let mut mem = Memory::new();

    // write a value into address 0; clk = 1
    mem.advance_clock();
    let addr0 = Felt::new(0);
    let value1 = [ONE, ZERO, ZERO, ZERO];
    mem.write(addr0, value1);
    assert_eq!(value1, mem.get_value(addr0.as_int()).unwrap());
    assert_eq!(1, mem.size());
    assert_eq!(1, mem.trace_len());

    // write a value into address 2; clk = 2
    mem.advance_clock();
    let addr2 = Felt::new(2);
    let value5 = [Felt::new(5), ZERO, ZERO, ZERO];
    mem.write(addr2, value5);
    assert_eq!(value5, mem.get_value(addr2.as_int()).unwrap());
    assert_eq!(2, mem.size());
    assert_eq!(2, mem.trace_len());

    // write a value into address 1; clk = 3
    mem.advance_clock();
    let addr1 = Felt::new(1);
    let value7 = [Felt::new(7), ZERO, ZERO, ZERO];
    mem.write(addr1, value7);
    assert_eq!(value7, mem.get_value(addr1.as_int()).unwrap());
    assert_eq!(3, mem.size());
    assert_eq!(3, mem.trace_len());

    // write a value into address 0; clk = 4
    mem.advance_clock();
    let value9 = [Felt::new(9), ZERO, ZERO, ZERO];
    mem.write(addr0, value9);
    assert_eq!(value7, mem.get_value(addr1.as_int()).unwrap());
    assert_eq!(3, mem.size());
    assert_eq!(4, mem.trace_len());

    // check generated trace and memory data provided to the ChipletsBus; rows should be sorted by
    // address and then clock cycle
    let (trace, chiplets_bus) = build_trace(mem, 4);

    // address 0
    let mut prev_row = [ZERO; MEMORY_TRACE_WIDTH];
    let memory_access = MemoryLookup::new(addr0, 1, [ZERO; 4], value1);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 0, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr0, 4, value1, value9);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 1, &memory_access, prev_row);

    // address 1
    let memory_access = MemoryLookup::new(addr1, 3, [ZERO; 4], value7);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 2, &memory_access, prev_row);

    // address 2
    let memory_access = MemoryLookup::new(addr2, 2, [ZERO; 4], value5);
    verify_memory_access(&trace, &chiplets_bus, 3, &memory_access, prev_row);
}

#[test]
fn mem_write_read() {
    let mut mem = Memory::new();

    // write 1 into address 5; clk = 1
    mem.advance_clock();
    let addr5 = Felt::new(5);
    let value1 = [ONE, ZERO, ZERO, ZERO];
    mem.write(addr5, value1);

    // write 4 into address 2; clk = 2
    mem.advance_clock();
    let addr2 = Felt::new(2);
    let value4 = [Felt::new(4), ZERO, ZERO, ZERO];
    mem.write(addr2, value4);

    // read a value from address 5; clk = 3
    mem.advance_clock();
    let _ = mem.read(addr5);

    // write 2 into address 5; clk = 4
    mem.advance_clock();
    let value2 = [Felt::new(2), ZERO, ZERO, ZERO];
    mem.write(addr5, value2);

    // read a value from address 2; clk = 5
    mem.advance_clock();
    let _ = mem.read(addr2);

    // write 7 into address 2; clk = 6
    mem.advance_clock();
    let value7 = [Felt::new(7), ZERO, ZERO, ZERO];
    mem.write(addr2, value7);

    // read a value from address 5; clk = 7
    mem.advance_clock();
    let _ = mem.read(addr5);

    // read a value from address 2; clk = 8
    mem.advance_clock();
    let _ = mem.read(addr2);

    // read a value from address 5; clk = 9
    mem.advance_clock();
    let _ = mem.read(addr5);

    // check generated trace and memory data provided to the ChipletsBus; rows should be sorted by
    // address and then clock cycle
    let (trace, chiplets_bus) = build_trace(mem, 9);

    // address 2
    let mut prev_row = [ZERO; MEMORY_TRACE_WIDTH];
    let memory_access = MemoryLookup::new(addr2, 2, [ZERO; 4], value4);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 0, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr2, 5, value4, value4);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 1, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr2, 6, value4, value7);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 2, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr2, 8, value7, value7);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 3, &memory_access, prev_row);

    // address 5
    let memory_access = MemoryLookup::new(addr5, 1, [ZERO; 4], value1);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 4, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr5, 3, value1, value1);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 5, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr5, 4, value1, value2);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 6, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr5, 7, value2, value2);
    prev_row = verify_memory_access(&trace, &chiplets_bus, 7, &memory_access, prev_row);

    let memory_access = MemoryLookup::new(addr5, 9, value2, value2);
    verify_memory_access(&trace, &chiplets_bus, 8, &memory_access, prev_row);
}

#[test]
fn mem_get_values_at() {
    let mut mem = Memory::new();

    // Write 1 into address 5 at clk = 1.
    // This means that mem[5] = 1 at the beginning of `clk=2`
    mem.advance_clock();
    let addr5 = Felt::new(5);
    let value1 = [ONE, ZERO, ZERO, ZERO];
    mem.write(addr5, value1);

    // Write 4 into address 2 at clk = 2.
    // This means that mem[4] = 2 at the beginning of `clk=3`
    mem.advance_clock();
    let addr2 = Felt::new(2);
    let value4 = [Felt::new(4), ZERO, ZERO, ZERO];
    mem.write(addr2, value4);

    mem.advance_clock();
    // Check that mem[5] == 1 at clk = 2.
    assert_eq!(vec![(5_u64, value1)], mem.get_values_at(0..=5, 2));

    // Check that mem[5] == 1 and mem[4] == 2 at clk = 3 and 4
    assert_eq!(
        vec![(2_u64, value4), (5_u64, value1)],
        mem.get_values_at(0..=5, 3)
    );
    assert_eq!(
        vec![(2_u64, value4), (5_u64, value1)],
        mem.get_values_at(0..=5, 4)
    );

    // Check that range works as expected
    assert_eq!(vec![(2_u64, value4)], mem.get_values_at(0..=4, 4));
}

// HELPER STRUCT & FUNCTIONS
// ================================================================================================

/// Builds a trace of the specified length and fills it with data from the provided Memory instance.
fn build_trace(mem: Memory, num_rows: usize) -> (Vec<Vec<Felt>>, ChipletsBus) {
    let mut chiplets_bus = ChipletsBus::default();
    let mut trace = (0..MEMORY_TRACE_WIDTH)
        .map(|_| vec![Felt::ZERO; num_rows])
        .collect::<Vec<_>>();
    let mut fragment = TraceFragment::trace_to_fragment(&mut trace);
    mem.fill_trace(&mut fragment, 0, &mut chiplets_bus);

    (trace, chiplets_bus)
}

fn read_trace_row(trace: &[Vec<Felt>], step: usize) -> [Felt; MEMORY_TRACE_WIDTH] {
    let mut row = [ZERO; MEMORY_TRACE_WIDTH];
    for (value, column) in row.iter_mut().zip(trace) {
        *value = column[step];
    }
    row
}

fn build_trace_row(
    memory_access: &MemoryLookup,
    prev_row: [Felt; MEMORY_TRACE_WIDTH],
) -> [Felt; MEMORY_TRACE_WIDTH] {
    let MemoryLookup {
        ctx,
        addr,
        clk,
        old_word: old_val,
        new_word: new_val,
    } = *memory_access;

    let mut row = [ZERO; MEMORY_TRACE_WIDTH];
    row[0] = ctx; // ctx
    row[1] = addr;
    row[2] = Felt::new(clk);
    row[3] = old_val[0];
    row[4] = old_val[1];
    row[5] = old_val[2];
    row[6] = old_val[3];
    row[7] = new_val[0];
    row[8] = new_val[1];
    row[9] = new_val[2];
    row[10] = new_val[3];

    if prev_row != [ZERO; MEMORY_TRACE_WIDTH] {
        let delta = if row[0] != prev_row[0] {
            row[0] - prev_row[0]
        } else if row[1] != prev_row[1] {
            row[1] - prev_row[1]
        } else {
            row[2] - prev_row[2] - ONE
        };

        let (hi, lo) = super::split_element_u32_into_u16(delta);
        row[11] = lo;
        row[12] = hi;
        row[13] = delta.inv();
    }

    row
}

fn verify_memory_access(
    trace: &[Vec<Felt>],
    chiplets_bus: &ChipletsBus,
    row: usize,
    memory_access: &MemoryLookup,
    prev_row: [Felt; MEMORY_TRACE_WIDTH],
) -> [Felt; MEMORY_TRACE_WIDTH] {
    let expected_row = build_trace_row(memory_access, prev_row);
    let expected_lookup = ChipletsLookupRow::Memory(*memory_access);
    let expected_hint = ChipletsLookup::Response(row);

    let lookup = chiplets_bus.get_response_row(row);
    let hint = chiplets_bus.get_lookup_hint(row).unwrap();

    assert_eq!(expected_row, read_trace_row(trace, row));
    assert_eq!(expected_lookup, lookup);
    assert_eq!(&expected_hint, hint);

    expected_row
}
