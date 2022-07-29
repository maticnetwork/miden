use crate::build_test;

// SIMPLE FLOW CONTROL TESTS
// ================================================================================================

#[test]
fn conditional_execution() {
    // --- if without else ------------------------------------------------------------------------
    let source = "begin dup.1 dup.1 eq if.true add end end";

    let test = build_test!(source, &[1, 2]);
    test.expect_stack(&[2, 1]);

    let test = build_test!(source, &[3, 3]);
    test.expect_stack(&[6]);

    // --- if with else ------------------------------------------------------------------------
    let source = "begin dup.1 dup.1 eq if.true add else mul end end";

    let test = build_test!(source, &[2, 3]);
    test.expect_stack(&[6]);

    let test = build_test!(source, &[3, 3]);
    test.expect_stack(&[6]);
}

#[test]
fn conditional_loop() {
    // --- entering the loop ----------------------------------------------------------------------
    // computes sum of values from 0 to the value at the top of the stack
    let source = "
        begin
            dup push.0 movdn.2 neq.0
            while.true
                dup movup.2 add swap push.1 sub dup neq.0
            end
            drop
        end";

    let test = build_test!(source, &[10]);
    test.expect_stack(&[55]);

    // --- skipping the loop ----------------------------------------------------------------------
    let source = "begin dup eq.0 while.true add end end";

    let test = build_test!(source, &[10]);
    test.expect_stack(&[10]);
}

#[test]
fn counter_controlled_loop() {
    // --- entering the loop ----------------------------------------------------------------------
    // compute 2^10
    let source = "
        begin
            push.2
            push.1
            repeat.10
                dup.1 mul
            end
            swap drop
        end";

    let test = build_test!(source);
    test.expect_stack(&[1024]);
}

// NESTED CONTROL FLOW
// ================================================================================================

#[test]
fn if_in_loop() {
    let source = "
        begin
            dup push.0 movdn.2 neq.0
            while.true
                dup movup.2 dup.1 eq.5
                if.true 
                    mul
                else
                    add
                end
                swap push.1 sub dup neq.0
            end
            drop
        end";

    let test = build_test!(source, &[10]);
    test.expect_stack(&[210]);
}

#[test]
fn if_in_loop_in_if() {
    let source = "
        begin
            dup eq.10
            if.true
                dup push.0 movdn.2 neq.0
                while.true
                    dup movup.2 dup.1 eq.5
                    if.true 
                        mul
                    else
                        add
                    end
                    swap push.1 sub dup neq.0
                end
                drop
            else
                dup mul
            end
        end";

    let test = build_test!(source, &[10]);
    test.expect_stack(&[210]);

    let test = build_test!(source, &[11]);
    test.expect_stack(&[121]);
}
