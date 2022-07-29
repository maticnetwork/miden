use super::{
    field_ops::append_pow2_op, parse_u32_param, push_value, AssemblyError, Felt, Operation, Token,
    Vec,
};

// ENUMS
// ================================================================================================

/// This enum is intended to determine the mode of operation passed to the parsing function
#[derive(PartialEq)]
pub enum U32OpMode {
    Checked,
    Unchecked,
    Wrapping,
    Overflowing,
}

// CONVERSIONS AND TESTS
// ================================================================================================

/// Translates u32test assembly instruction to VM operations.
///
/// Implemented as: `DUP U32SPLIT SWAP DROP EQZ` (5 VM cycles).
pub fn parse_u32test(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            span_ops.push(Operation::Dup0);
            span_ops.push(Operation::U32split);
            span_ops.push(Operation::Swap);
            span_ops.push(Operation::Drop);
            span_ops.push(Operation::Eqz);
        }
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32testw assembly instruction to VM operations.
///
/// Implemented by executing DUP U32SPLIT SWAP DROP EQZ on each element in the word
/// and combining the results using AND operation (total of 23 VM cycles)
pub fn parse_u32testw(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            // Test the fourth element
            span_ops.push(Operation::Dup3);
            span_ops.push(Operation::U32split);
            span_ops.push(Operation::Swap);
            span_ops.push(Operation::Drop);
            span_ops.push(Operation::Eqz);

            // Test the third element
            span_ops.push(Operation::Dup3);
            span_ops.push(Operation::U32split);
            span_ops.push(Operation::Swap);
            span_ops.push(Operation::Drop);
            span_ops.push(Operation::Eqz);
            span_ops.push(Operation::And);

            // Test the second element
            span_ops.push(Operation::Dup2);
            span_ops.push(Operation::U32split);
            span_ops.push(Operation::Swap);
            span_ops.push(Operation::Drop);
            span_ops.push(Operation::Eqz);
            span_ops.push(Operation::And);

            // Test the first element
            span_ops.push(Operation::Dup1);
            span_ops.push(Operation::U32split);
            span_ops.push(Operation::Swap);
            span_ops.push(Operation::Drop);
            span_ops.push(Operation::Eqz);
            span_ops.push(Operation::And);
        }
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32assert assembly instruction to VM operations.
///
/// u32assert, u32assert.1: Implemented as: `PAD U32ASSERT2 DROP` (3 VM cycles).
/// u32assert.2: Implemented as: `U32assert2` (1 VM cycles).
pub fn parse_u32assert(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => assert_u32(span_ops),
        2 => match op.parts()[1] {
            "1" => assert_u32(span_ops),
            "2" => span_ops.push(Operation::U32assert2),
            _ => return Err(AssemblyError::invalid_param(op, 1)),
        },
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32assertw assembly instruction to VM operations.
///
/// Implemented by executing `U32ASSERT2` on each pair of elements in the word.
/// Total of 6 VM cycles.
pub fn parse_u32assertw(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            // Test the first and the second elements
            span_ops.push(Operation::U32assert2);

            // Move 3 and 4 to the top of the stack
            span_ops.push(Operation::MovUp3);
            span_ops.push(Operation::MovUp3);

            // Test them
            span_ops.push(Operation::U32assert2);

            // Move the elements back into place
            span_ops.push(Operation::MovUp3);
            span_ops.push(Operation::MovUp3);
        }
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32cast assembly instruction to VM operations.
///
/// Implemented as: `U32SPLIT DROP` (2 VM cycles).
pub fn parse_u32cast(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            span_ops.push(Operation::U32split);
            span_ops.push(Operation::Drop);
        }
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32split assembly instruction to VM operations.
///
/// Implement as: `U32SPLIT` (1 VM cycle).
pub fn parse_u32split(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => span_ops.push(Operation::U32split),
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

// ARITHMETIC OPERATIONS
// ================================================================================================

/// Translates u32add assembly instructions to VM operations.
///
/// The base operation is `U32ADD`, but depending on the mode, additional operations may be
/// inserted. Please refer to the docs of `handle_arithmetic_operation` for more details.
///
/// VM cycles per mode:
/// - u32checked_add: 4 cycles
/// - u32checked_add.b:
///    - 6 cycles if b = 1
///    - 5 cycles if b != 1
/// - u32wrapping_add: 2 cycles
/// - u32wrapping_add.b: 3 cycles
/// - u32overflowing_add: 1 cycles
/// - u32overflowing_add.b: 2 cycles
pub fn parse_u32add(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_arithmetic_operation(span_ops, op, Operation::U32add, op_mode)
}

/// Translates u32overflowing_add3 assembly instruction directly to `U32ADD3` operation.
///
/// This operation takes 1 VM cycle.
pub fn parse_u32add3(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            if op_mode != U32OpMode::Overflowing {
                return Err(AssemblyError::invalid_op(op));
            }
        }
        _ => return Err(AssemblyError::extra_param(op)),
    }

    span_ops.push(Operation::U32add3);

    Ok(())
}

/// Translates u32sub assembly instructions to VM operations.
///
/// The base operation is `U32SUB`, but depending on the mode, additional operations may be
/// inserted. Please refer to the docs of `handle_arithmetic_operation` for more details.
///
/// VM cycles per mode:
/// - u32checked_sub: 4 cycles
/// - u32checked_sub.b:
///    - 6 cycles if b = 1
///    - 5 cycles if b != 1
/// - u32wrapping_sub: 2 cycles
/// - u32wrapping_sub.b: 3 cycles
/// - u32overflowing_sub: 1 cycles
/// - u32overflowing_sub.b: 2 cycles
pub fn parse_u32sub(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_arithmetic_operation(span_ops, op, Operation::U32sub, op_mode)
}

/// Translates u32mul assembly instructions to VM operations.
///
/// The base operation is `U32MUL`, but depending on the mode, additional operations may be
/// inserted. Please refer to the docs of `handle_arithmetic_operation` for more details.
///
/// VM cycles per mode:
/// - u32checked_mul: 4 cycles
/// - u32checked_mul.b:
///    - 6 cycles if b = 1
///    - 5 cycles if b != 1
/// - u32wrapping_mul: 2 cycles
/// - u32wrapping_mul.b: 3 cycles
/// - u32overflowing_mul: 1 cycles
/// - u32overflowing_mul.b: 2 cycles
pub fn parse_u32mul(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_arithmetic_operation(span_ops, op, Operation::U32mul, op_mode)
}

/// Translates u32overflowing_madd assembly instruction directly to `U32MADD` operation.
///
/// This operation takes 1 VM cycle.
pub fn parse_u32madd(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            if op_mode != U32OpMode::Overflowing {
                return Err(AssemblyError::invalid_op(op));
            }
        }
        _ => return Err(AssemblyError::extra_param(op)),
    }

    span_ops.push(Operation::U32madd);

    Ok(())
}

/// Translates u32div assembly instructions to VM operations.
///
/// VM cycles per mode:
/// - u32checked_div: 3 cycles
/// - u32checked_div.b:
///    - 5 cycles if b is 1
///    - 4 cycles if b is not 1
/// - u32unchecked_div: 2 cycles
/// - u32unchecked_div.b:
///    - 4 cycles if b is 1
///    - 3 cycles if b is not 1
pub fn parse_u32div(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_division(span_ops, op, op_mode)?;

    span_ops.push(Operation::Drop);

    Ok(())
}

/// Translates u32mod assembly instructions to VM operations.
///
/// VM cycles per mode:
/// - u32checked_mod: 4 cycles
/// - u32checked_mod.b:
///    - 6 cycles if b is 1
///    - 5 cycles if b is not 1
/// - u32unchecked_mod: 3 cycle
/// - u32unchecked_mod.b:
///    - 5 cycles if b is 1
///    - 4 cycles if b is not 1
pub fn parse_u32mod(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_division(span_ops, op, op_mode)?;

    span_ops.push(Operation::Swap);
    span_ops.push(Operation::Drop);

    Ok(())
}

/// Translates u32divmod assembly instructions to VM operations.
///
/// VM cycles per mode:
/// - u32checked_divmod: 2 cycles
/// - u32checked_divmod.b:
///    - 4 cycles if b is 1
///    - 3 cycles if b is not 1
/// - u32unchecked_divmod: 1 cycle
/// - u32unchecked_divmod.b:
///    - 3 cycles if b is 1
///    - 2 cycles if b is not 1
pub fn parse_u32divmod(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_division(span_ops, op, op_mode)?;

    Ok(())
}

// BITWISE OPERATIONS
// ================================================================================================

/// Translates u32checked_and assembly instruction to VM operation.
///
/// Implemented as: `U32AND` (1 VM cycle).
///
/// We don't need to assert that inputs are u32 values because the VM does these assertions
/// implicitly for `U32AND` operation.
pub fn parse_u32and(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => span_ops.push(Operation::U32and),
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32checked_or assembly instruction to VM operation `U32OR`.
///
/// Implemented as: `U32OR` (1 VM cycle).
///
/// We don't need to assert that inputs are u32 values because the VM does these assertions
/// implicitly for `U32OR` operation.
pub fn parse_u32or(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => span_ops.push(Operation::U32or),
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32checked_xor assembly instruction to VM operation `U32XOR`.
///
/// Implemented as: `U32XOR` (1 VM cycle).
///
/// We don't need to assert that inputs are u32 values because the VM does these assertions
/// implicitly for `U32XOR` operation.
pub fn parse_u32xor(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => span_ops.push(Operation::U32xor),
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32checked_not assembly instruction to VM operations.
///
/// The operation is implemented as `PUSH(2^32 - 1) U32ASSERT2 SWAP U32SUB DROP`,
/// total to 5 cycles.
///
/// The reason this method works is because 2^32 provides a bit mask of ones, which after
/// subtracting the element, flips the bits of the original value to perform a bitwise NOT.
pub fn parse_u32not(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            // perform the operation
            span_ops.push(Operation::Push(Felt::new(2u64.pow(32) - 1)));
            span_ops.push(Operation::U32assert2);
            span_ops.push(Operation::Swap);
            span_ops.push(Operation::U32sub);

            // Drop the underflow flag
            span_ops.push(Operation::Drop);
        }
        _ => return Err(AssemblyError::extra_param(op)),
    }

    Ok(())
}

/// Translates u32shl assembly instructions to VM operations.
///
/// The operation is implemented by putting a power of 2 on the stack, then multiplying it with
/// the value to be shifted and splitting the result. For checked variants, the shift value is
/// asserted to be between 0-31 and the value to be shifted is asserted to be a 32-bit value.
///
/// VM cycles per mode:
/// - u32checked_shl: 47 cycles
/// - u32checked_shl.b: 4 cycles
/// - u32unchecked_shl: 40 cycles
/// - u32unchecked_shl.b: 3 cycles
pub fn parse_u32shl(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::invalid_op(op)),
        1 => match op_mode {
            U32OpMode::Checked => {
                // Assume the dynamic shift value b is on top of the stack.
                append_pow2_op(span_ops, true);
                span_ops.push(Operation::U32assert2);
            }
            U32OpMode::Unchecked => {
                append_pow2_op(span_ops, false);
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        2 => match op_mode {
            U32OpMode::Checked => {
                let x = parse_u32_param(op, 1, 0, 31)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(x))));
                span_ops.push(Operation::U32assert2);
            }
            U32OpMode::Unchecked => {
                let x = parse_u32_param(op, 1, 0, u32::MAX)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(x))));
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        _ => return Err(AssemblyError::extra_param(op)),
    }

    span_ops.push(Operation::U32mul);
    span_ops.push(Operation::Drop);

    Ok(())
}

/// Translates u32shr assembly instructions to VM operations.
///
/// The operation is implemented by putting a power of 2 on the stack, then dividing the value to
/// be shifted by it and returning the quotient. For checked variants, the shift value is asserted
/// to be between 0-31 and the value to be shifted is asserted to be a 32-bit value.
///
/// VM cycles per mode:
/// - u32checked_shr: 47 cycles
/// - u32checked_shr.b: 4 cycles
/// - u32unchecked_shr: 40 cycles
/// - u32unchecked_shr.b: 3 cycles
pub fn parse_u32shr(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::invalid_op(op)),
        1 => match op_mode {
            U32OpMode::Checked => {
                // Assume the dynamic shift value b is on top of the stack.
                append_pow2_op(span_ops, true);
                span_ops.push(Operation::U32assert2);
            }
            U32OpMode::Unchecked => {
                append_pow2_op(span_ops, false);
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        2 => match op_mode {
            U32OpMode::Checked => {
                let x = parse_u32_param(op, 1, 0, 31)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(x))));
                span_ops.push(Operation::U32assert2);
            }
            U32OpMode::Unchecked => {
                let x = parse_u32_param(op, 1, 0, u32::MAX)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(x))));
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        _ => return Err(AssemblyError::extra_param(op)),
    };

    // Use division to shift right and then drop the remainder
    span_ops.push(Operation::U32div);
    span_ops.push(Operation::Drop);

    Ok(())
}

/// Translates u32rotl assembly instructions to VM operations.
///
/// The base operation is implemented by putting a power of 2 on the stack, then multiplying the
/// value to be shifted by it and adding the overflow limb to the shifted limb. For the checked
/// variants, the shift value is asserted to be between 0-31 and the value to be shifted is
/// asserted to be a 32-bit value.
///
/// VM cycles per mode:
/// - u32checked_rotl: 47 cycles
/// - u32checked_rotl.b: 4 cycles
/// - u32unchecked_rotl: 40 cycles
/// - u32unchecked_rotl.b: 3 cycles
pub fn parse_u32rotl(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::invalid_op(op)),
        1 => match op_mode {
            U32OpMode::Checked => {
                // Assume the dynamic shift value b is on top of the stack.
                append_pow2_op(span_ops, true);
                span_ops.push(Operation::U32assert2);
            }
            U32OpMode::Unchecked => {
                append_pow2_op(span_ops, false);
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        2 => match op_mode {
            U32OpMode::Checked => {
                let x = parse_u32_param(op, 1, 0, 31)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(x))));
                span_ops.push(Operation::U32assert2);
            }
            U32OpMode::Unchecked => {
                let x = parse_u32_param(op, 1, 0, u32::MAX)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(x))));
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        _ => return Err(AssemblyError::extra_param(op)),
    }

    span_ops.push(Operation::U32mul);
    span_ops.push(Operation::Add);

    Ok(())
}

/// Translates u32rotr assembly instructions to VM operations.
///
/// The base operation is implemented by multiplying the value to be shifted by 2^(32-b), where
/// b is the shift amount, then adding the overflow limb to the shifted limb. For the checked
/// variants, the shift value is asserted to be between 0-31 and the value to be shifted is
/// asserted to be a 32-bit value.
///
/// VM cycles per mode:
/// - u32checked_rotr: 59 cycles
/// - u32checked_rotr.b: 6 cycles
/// - u32unchecked_rotr: 44 cycles
/// - u32unchecked_rotr.b: 3 cycles
pub fn parse_u32rotr(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::invalid_op(op)),
        1 => match op_mode {
            U32OpMode::Checked => {
                // Verify both b and a are u32.
                span_ops.push(Operation::U32assert2);
                // Calculate 32 - b and assert that the shift value b <= 31.
                span_ops.push(Operation::Push(Felt::new(31)));
                span_ops.push(Operation::Dup1);
                span_ops.push(Operation::U32sub);
                span_ops.push(Operation::Not);
                span_ops.push(Operation::Assert);
                span_ops.push(Operation::Incr);
                span_ops.push(Operation::Dup1);
                // If 32-b = 32, replace it with 0.
                span_ops.push(Operation::Eqz);
                span_ops.push(Operation::Not);
                span_ops.push(Operation::CSwap);
                span_ops.push(Operation::Drop);
                append_pow2_op(span_ops, true);
                span_ops.push(Operation::Swap);
            }
            U32OpMode::Unchecked => {
                span_ops.push(Operation::Push(Felt::new(32)));
                span_ops.push(Operation::Swap);
                span_ops.push(Operation::U32sub);
                span_ops.push(Operation::Drop);
                append_pow2_op(span_ops, false);
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        2 => match op_mode {
            U32OpMode::Checked => {
                // Assert the top of the stack is a u32 value.
                // NOTE: We cannot use U32Assert2 since we are potentially pushing a number larger
                // than u32 for b.
                assert_u32(span_ops);

                let x = parse_u32_param(op, 1, 0, 31)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(32 - x))));
            }
            U32OpMode::Unchecked => {
                let x = parse_u32_param(op, 1, 0, u32::MAX)?;
                span_ops.push(Operation::Push(Felt::new(2u64.pow(32 - x))));
            }
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        _ => return Err(AssemblyError::extra_param(op)),
    }

    span_ops.push(Operation::U32mul);
    span_ops.push(Operation::Add);

    Ok(())
}

// COMPARISON OPERATIONS
// ================================================================================================

/// Translates u32checked_eq assembly instruction to VM operations.
///
/// Specifically we test the first two numbers to be u32, then perform a EQ to check the equality.
///
/// VM cycles per mode:
/// - u32checked_eq: 2 cycles
/// - u32checked_eq.b: 3 cycles
pub fn parse_u32eq(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => span_ops.push(Operation::U32assert2),
        2 => assert_and_push_u32_param(span_ops, op, 0)?,
        _ => return Err(AssemblyError::extra_param(op)),
    }

    span_ops.push(Operation::Eq);

    Ok(())
}

/// Translates u32checked_neq assembly instruction to VM operations.
///
/// Specifically we test the first two numbers to be u32, then perform a `EQ NOT` to check the
/// equality.
///
/// VM cycles per mode:
/// - u32checked_neq: 3 cycles
/// - u32checked_neq.b: 4 cycles
pub fn parse_u32neq(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => span_ops.push(Operation::U32assert2),
        2 => assert_and_push_u32_param(span_ops, op, 0)?,
        _ => return Err(AssemblyError::extra_param(op)),
    }

    span_ops.push(Operation::Eq);
    span_ops.push(Operation::Not);

    Ok(())
}

/// Translates u32lt assembly instructions to VM operations.
///
/// Specifically we test the first two numbers to be u32, then perform a `U32SUB EQZ NOT` to check
/// the underflow flag.
///
/// VM cycles per mode:
/// - u32checked_lt: 6 cycles
/// - u32unchecked_lt 5 cycles
pub fn parse_u32lt(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_u32_and_unchecked_mode(span_ops, op, op_mode)?;
    compute_lt(span_ops);

    Ok(())
}

/// Translates u32lte assembly instructions to VM operations.
///
/// Specifically we test the first two numbers to be u32, then perform a gt check and flip the
/// results.
///
/// VM cycles per mode:
/// - u32checked_lte: 8 cycles
/// - u32unchecked_lte: 7 cycles
pub fn parse_u32lte(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_u32_and_unchecked_mode(span_ops, op, op_mode)?;

    // Compute the lt with reversed number to get a gt check
    span_ops.push(Operation::Swap);
    compute_lt(span_ops);

    // Flip the final results to get the lte results.
    span_ops.push(Operation::Not);

    Ok(())
}

/// Translates u32gt assembly instructions to VM operations.
///
/// Specifically we test the first two numbers to be u32, then perform a lt check with the
/// numbers swapped.
///
/// VM cycles per mode:
/// - u32checked_gt: 7 cycles
/// - u32unchecked_gt: 6 cycles
pub fn parse_u32gt(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_u32_and_unchecked_mode(span_ops, op, op_mode)?;

    // Reverse the numbers so we can get a gt check.
    span_ops.push(Operation::Swap);

    compute_lt(span_ops);

    Ok(())
}

/// Translates u32gte assembly instructions to VM operations.
///
/// Specifically we test the first two numbers to be u32, then compute a lt check and flip the
/// results.
///
/// VM cycles per mode:
/// - u32checked_gte: 7 cycles
/// - u32unchecked_gte: 6 cycles
pub fn parse_u32gte(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    handle_u32_and_unchecked_mode(span_ops, op, op_mode)?;

    compute_lt(span_ops);

    // Flip the final results to get the gte results.
    span_ops.push(Operation::Not);

    Ok(())
}

/// Translates u32min assembly instructions to VM operations.
///
/// Specifically, we test the first two numbers to be u32 (U32SPLIT NOT ASSERT), subtract the top
/// value from the second to the top value (U32SUB), check the underflow flag (EQZ), and perform a
/// conditional swap (CSWAP) to have the max number in front. Then we finally drop the top element
/// to keep the min.
///
/// VM cycles per mode:
/// - u32checked_min: 9 cycles
/// - u32unchecked_min: 8 cycles
pub fn parse_u32min(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => match op_mode {
            U32OpMode::Checked | U32OpMode::Unchecked => (),
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        _ => return Err(AssemblyError::extra_param(op)),
    }

    compute_max_and_min(span_ops, op_mode);

    // Drop the max and keep the min
    span_ops.push(Operation::Drop);

    Ok(())
}

/// Translates u32max assembly instructions to VM operations.
///
/// Specifically, we test the first two values to be u32 (U32SPLIT NOT ASSERT), subtract the top
/// value from the second to the top value (U32SUB), check the underflow flag (EQZ), and perform
/// a conditional swap (CSWAP) to have the max number in front. then we finally drop the 2nd
/// element to keep the max.
///
/// VM cycles per mode:
/// - u32checked_max: 10 cycles
/// - u32unchecked_max: 9 cycles
pub fn parse_u32max(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => match op_mode {
            U32OpMode::Checked | U32OpMode::Unchecked => (),
            _ => return Err(AssemblyError::invalid_op(op)),
        },
        _ => return Err(AssemblyError::extra_param(op)),
    }

    compute_max_and_min(span_ops, op_mode);

    // Drop the min and keep the max
    span_ops.push(Operation::Swap);
    span_ops.push(Operation::Drop);

    Ok(())
}

// HELPER FUNCTIONS
// ================================================================================================
/// Asserts that the value on the top of the stack is a u32.
///
/// Implemented as: `PAD U32ASSERT2 DROP`.
///
/// This operation takes 3 VM cycles.
fn assert_u32(span_ops: &mut Vec<Operation>) {
    span_ops.push(Operation::Pad);
    span_ops.push(Operation::U32assert2);
    span_ops.push(Operation::Drop);
}

/// Asserts that the value on the top of the stack is a u32 and pushes the first param of the `op`
/// as a u32 value onto the stack.
///
/// This operation takes:
/// - 3 VM cycles when the param == 1.
/// - 2 VM cycle when the param != 1.
///
/// # Errors
/// Returns an error if the first parameter of the `op` is not a u32 value or is greater than
/// `lower_bound`.
fn assert_and_push_u32_param(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    lower_bound: u32,
) -> Result<(), AssemblyError> {
    // TODO: We should investigate special case handling adding 0 or 1.
    let value = parse_u32_param(op, 1, lower_bound, u32::MAX)?;
    push_value(span_ops, Felt::new(value as u64));

    // Assert both numbers are u32.
    span_ops.push(Operation::U32assert2);

    Ok(())
}

/// Pushes the first param of the `op` onto the stack. The param is not checked for
/// belonging to u32.
///
/// This operation takes:
/// - 2 VM cycles when the param == 1.
/// - 1 VM cycle when the param != 1.
///
/// # Errors
/// Returns an error if we try to push 0 as a divisor.
fn push_int_param(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
    is_divisor: bool,
) -> Result<(), AssemblyError> {
    let param_value = op.parts()[1];

    // attempt to parse the parameter value as an integer
    let value = match param_value.parse::<u64>() {
        Ok(i) => i,
        Err(_) => return Err(AssemblyError::invalid_param(op, 1)),
    };

    if value > (u32::MAX as u64) && op_mode == U32OpMode::Checked {
        return Err(AssemblyError::invalid_param(op, 1));
    }

    if is_divisor && value == 0 {
        return Err(AssemblyError::invalid_param_with_reason(
            op,
            1,
            "division by 0",
        ));
    }

    push_value(span_ops, Felt::new(value));

    Ok(())
}

/// Duplicate the top two elements in the stack and check both are u32, and determine the min
/// and max between them.
///
/// The maximum number will be at the top of the stack and minimum will be at the 2nd index.
fn compute_max_and_min(span_ops: &mut Vec<Operation>, op_mode: U32OpMode) {
    // Copy top two elements of the stack.
    span_ops.push(Operation::Dup1);
    span_ops.push(Operation::Dup1);
    if op_mode == U32OpMode::Checked {
        span_ops.push(Operation::U32assert2);
    }

    span_ops.push(Operation::U32sub);
    span_ops.push(Operation::Swap);
    span_ops.push(Operation::Drop);

    // Check the underflow flag, if it's zero
    // then the second number is equal or larger than the first.
    span_ops.push(Operation::Eqz);
    span_ops.push(Operation::CSwap);
}

/// Inserts the VM operations to check if the second element is less than
/// the top element. This takes 5 cycles.
fn compute_lt(span_ops: &mut Vec<Operation>) {
    span_ops.push(Operation::U32sub);
    span_ops.push(Operation::Swap);
    span_ops.push(Operation::Drop);

    // Check the underflow flag
    span_ops.push(Operation::Eqz);
    span_ops.push(Operation::Not);
}

/// Handles u32 assertion and unchecked mode for any u32 operation.
fn handle_u32_and_unchecked_mode(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op.num_parts() {
        0 => return Err(AssemblyError::missing_param(op)),
        1 => {
            if op_mode == U32OpMode::Checked {
                span_ops.push(Operation::U32assert2);
            }
        }
        _ => return Err(AssemblyError::extra_param(op)),
    };

    Ok(())
}

/// Handles U32ADD, U32SUB, and U32MUL operations in checked, wrapping, and overflowing modes,
/// including handling of immediate parameters.
///
/// Specifically handles these specific inputs per the spec.
/// - Checked: fails if either of the inputs or the output is not a u32 value.
/// - Wrapping: does not check if the inputs are u32 values (the immediate value is also not
///   checked); overflow or underflow bits are discarded.
/// - Overflowing: does not check if the inputs are u32 values (the immediate value is also not
///   checked); overflow or underflow bits are pushed onto the stack.
fn handle_arithmetic_operation(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    arithmetic_op: Operation,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    let mut drop_high_bits = false;
    let mut assert_u32_res = false;
    let num_parts = op.num_parts();

    match op_mode {
        U32OpMode::Checked => {
            if num_parts == 1 {
                span_ops.push(Operation::U32assert2);
            } else {
                assert_and_push_u32_param(span_ops, op, 0)?;
            }
            assert_u32_res = true;
        }
        U32OpMode::Wrapping => {
            if num_parts == 2 {
                push_int_param(span_ops, op, U32OpMode::Wrapping, false)?;
            }
            drop_high_bits = true;
        }
        U32OpMode::Overflowing => {
            if num_parts == 2 {
                push_int_param(span_ops, op, U32OpMode::Overflowing, false)?;
            }
        }
        _ => return Err(AssemblyError::invalid_op(op)),
    }

    span_ops.push(arithmetic_op);

    if assert_u32_res {
        span_ops.push(Operation::Eqz);
        span_ops.push(Operation::Assert);
    } else if drop_high_bits {
        span_ops.push(Operation::Drop);
    }

    Ok(())
}

/// Handles common parts of u32div, u32mod, and u32divmod operations in checked and unchecked modes,
/// including handling of immediate parameters.
fn handle_division(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    op_mode: U32OpMode,
) -> Result<(), AssemblyError> {
    match op_mode {
        U32OpMode::Checked => {
            if op.num_parts() == 2 {
                push_int_param(span_ops, op, U32OpMode::Checked, true)?;
            }
            span_ops.push(Operation::U32assert2);
        }
        U32OpMode::Unchecked => {
            if op.num_parts() == 2 {
                push_int_param(span_ops, op, U32OpMode::Unchecked, true)?;
            }
        }
        _ => return Err(AssemblyError::invalid_op(op)),
    }

    span_ops.push(Operation::U32div);

    Ok(())
}
