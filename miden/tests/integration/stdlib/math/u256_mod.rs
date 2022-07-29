use super::build_test;
use num_bigint::BigUint;
use rand_utils::rand_vector;

// MULTIPLICATION
// ================================================================================================

#[test]
fn mul_unsafe() {
    let a = rand_u256();
    let b = rand_u256();

    let source = "
        use.std::math::u256
        begin
            exec.u256::mul_unsafe
        end";

    let operands = a
        .to_u32_digits()
        .iter()
        .chain(b.to_u32_digits().iter())
        .map(|&v| v as u64)
        .collect::<Vec<_>>();
    let result = (a * b)
        .to_u32_digits()
        .iter()
        .map(|&v| v as u64)
        .take(8)
        .rev()
        .collect::<Vec<_>>();

    build_test!(source, &operands).expect_stack(&result);
}

// HELPER FUNCTIONS
// ================================================================================================

fn rand_u256() -> BigUint {
    let limbs = rand_vector::<u64>(8)
        .iter()
        .map(|&v| v as u32)
        .collect::<Vec<_>>();
    BigUint::new(limbs)
}
