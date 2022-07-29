use crate::build_test;
use rand_utils::rand_value;

// ADVICE INJECTION
// ================================================================================================

#[test]
fn advice_inject_u64div() {
    let source = "begin adv.u64div push.adv.4 end";

    // get two random 64-bit integers and split them into 32-bit limbs
    let a = rand_value::<u64>();
    let a_hi = a >> 32;
    let a_lo = a as u32 as u64;

    let b = rand_value::<u64>();
    let b_hi = b >> 32;
    let b_lo = b as u32 as u64;

    // compute expected quotient
    let q = a / b;
    let q_hi = q >> 32;
    let q_lo = q as u32 as u64;

    // compute expected remainder
    let r = a % b;
    let r_hi = r >> 32;
    let r_lo = r as u32 as u64;

    // inject a/b into the advice tape and then read these values from the tape
    let test = build_test!(source, &[a_lo, a_hi, b_lo, b_hi]);
    let expected = [r_hi, r_lo, q_hi, q_lo, b_hi, b_lo, a_hi, a_lo];
    test.expect_stack(&expected);
}

#[test]
fn advice_inject_u64div_repeat() {
    // This procedure repeats the following steps 7 times:
    // - pushes quotient and remainder to advice tape
    // - drops divisor (top 2 elements of the stack reperesenting 32 bit limbs of divisor)
    // - reads quotient from advice tape to the stack
    // - push 2_u64 to the stack divided into 2 32 bit limbs
    // Finally the first 2 elements of the stack are removed
    let source = "begin 
        repeat.7 
            adv.u64div 
            drop drop
            push.adv.2
            push.2
            push.0
        end
        drop drop
    end";

    let mut a = 256;
    let a_hi = 0;
    let a_lo = a;

    let b = 2;
    let b_hi = 0;
    let b_lo = b;

    let mut expected = vec![a_lo, a_hi];

    for _ in 0..7 {
        let q = a / b;
        let q_hi = 0;
        let q_lo = q;
        expected.extend_from_slice(&[q_lo, q_hi]);
        a = q;
    }

    expected.reverse();

    let test = build_test!(source, &[a_lo, a_hi, b_lo, b_hi]);
    test.expect_stack(&expected);
}

#[test]
fn advice_inject_u64div_local_procedure() {
    let source = "proc.foo adv.u64div push.adv.4 end begin exec.foo end";

    // get two random 64-bit integers and split them into 32-bit limbs
    let a = rand_value::<u64>();
    let a_hi = a >> 32;
    let a_lo = a as u32 as u64;

    let b = rand_value::<u64>();
    let b_hi = b >> 32;
    let b_lo = b as u32 as u64;

    // compute expected quotient
    let q = a / b;
    let q_hi = q >> 32;
    let q_lo = q as u32 as u64;

    // compute expected remainder
    let r = a % b;
    let r_hi = r >> 32;
    let r_lo = r as u32 as u64;

    // inject a/b into the advice tape and then read these values from the tape
    let test = build_test!(source, &[a_lo, a_hi, b_lo, b_hi]);
    let expected = [r_hi, r_lo, q_hi, q_lo, b_hi, b_lo, a_hi, a_lo];
    test.expect_stack(&expected);
}

#[test]
fn advice_inject_u64div_conditional_execution() {
    let source = "begin eq if.true adv.u64div push.adv.4 else padw end end";

    // if branch
    let test = build_test!(source, &[8, 0, 4, 0, 1, 1]);
    test.expect_stack(&[0, 0, 0, 2, 0, 4, 0, 8]);

    // else branch
    let test = build_test!(source, &[8, 0, 4, 0, 1, 0]);
    test.expect_stack(&[0, 0, 0, 0, 0, 4, 0, 8]);
}
