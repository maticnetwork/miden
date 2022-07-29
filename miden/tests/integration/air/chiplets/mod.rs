use crate::build_test;
use rand_utils::rand_vector;

mod bitwise;
mod hasher;
mod memory;

#[test]
fn chiplets() {
    // Test a program that uses all of the chiplets.
    let source = "begin
        rpperm                          # hasher operation
        push.5 push.10 u32checked_or    # bitwise operation
        push.mem                        # memory operation
        drop                            # make sure the stack overflow table is empty
    end";
    let pub_inputs = rand_vector::<u64>(8);

    build_test!(source, &pub_inputs).prove_and_verify(pub_inputs, 0, false);
}
