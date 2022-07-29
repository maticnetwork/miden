use super::{AssemblyContext, AssemblyError, CodeBlock, Token, TokenStream};
pub use blocks::{combine_blocks, parse_code_blocks};
use u32_ops::U32OpMode;
use vm_core::{
    utils::{
        collections::Vec,
        string::{String, ToString},
    },
    AssemblyOp, Decorator, DecoratorList, Felt, FieldElement, Operation, StarkField,
};

mod blocks;
mod crypto_ops;
mod field_ops;
mod io_ops;
mod stack_ops;
mod u32_ops;

// OP PARSER
// ================================================================================================

/// Transforms an assembly instruction into a sequence of one or more VM instructions.
fn parse_op_token(
    op: &Token,
    span_ops: &mut Vec<Operation>,
    num_proc_locals: u32,
    decorators: &mut DecoratorList,
    in_debug_mode: bool,
) -> Result<(), AssemblyError> {
    let dec_len = decorators.len();
    // if assembler is in debug mode, populate decorators list with debug related
    // decorators like AsmOp.
    if in_debug_mode {
        decorators.push((
            span_ops.len(),
            Decorator::AsmOp(AssemblyOp::new(op.to_string(), 1)),
        ));
    }

    // based on the instruction, invoke the correct parser for the operation
    match op.parts()[0] {
        // ----- field operations -----------------------------------------------------------------
        "assert" => field_ops::parse_assert(span_ops, op),

        "add" => field_ops::parse_add(span_ops, op),
        "sub" => field_ops::parse_sub(span_ops, op),
        "mul" => field_ops::parse_mul(span_ops, op),
        "div" => field_ops::parse_div(span_ops, op),
        "neg" => field_ops::parse_neg(span_ops, op),
        "inv" => field_ops::parse_inv(span_ops, op),

        "checked_pow2" => field_ops::parse_pow2(span_ops, op, true),
        "unchecked_pow2" => field_ops::parse_pow2(span_ops, op, false),

        "not" => field_ops::parse_not(span_ops, op),
        "and" => field_ops::parse_and(span_ops, op),
        "or" => field_ops::parse_or(span_ops, op),
        "xor" => field_ops::parse_xor(span_ops, op),

        "eq" => field_ops::parse_eq(span_ops, op),
        "neq" => field_ops::parse_neq(span_ops, op),
        "lt" => field_ops::parse_lt(span_ops, op),
        "lte" => field_ops::parse_lte(span_ops, op),
        "gt" => field_ops::parse_gt(span_ops, op),
        "gte" => field_ops::parse_gte(span_ops, op),
        "eqw" => field_ops::parse_eqw(span_ops, op),

        // ----- u32 operations -------------------------------------------------------------------
        "u32test" => u32_ops::parse_u32test(span_ops, op),
        "u32testw" => u32_ops::parse_u32testw(span_ops, op),
        "u32assert" => u32_ops::parse_u32assert(span_ops, op),
        "u32assertw" => u32_ops::parse_u32assertw(span_ops, op),
        "u32cast" => u32_ops::parse_u32cast(span_ops, op),
        "u32split" => u32_ops::parse_u32split(span_ops, op),

        "u32checked_add" => u32_ops::parse_u32add(span_ops, op, U32OpMode::Checked),
        "u32wrapping_add" => u32_ops::parse_u32add(span_ops, op, U32OpMode::Wrapping),
        "u32overflowing_add" => u32_ops::parse_u32add(span_ops, op, U32OpMode::Overflowing),

        "u32overflowing_add3" => u32_ops::parse_u32add3(span_ops, op, U32OpMode::Overflowing),

        "u32checked_sub" => u32_ops::parse_u32sub(span_ops, op, U32OpMode::Checked),
        "u32wrapping_sub" => u32_ops::parse_u32sub(span_ops, op, U32OpMode::Wrapping),
        "u32overflowing_sub" => u32_ops::parse_u32sub(span_ops, op, U32OpMode::Overflowing),

        "u32checked_mul" => u32_ops::parse_u32mul(span_ops, op, U32OpMode::Checked),
        "u32wrapping_mul" => u32_ops::parse_u32mul(span_ops, op, U32OpMode::Wrapping),
        "u32overflowing_mul" => u32_ops::parse_u32mul(span_ops, op, U32OpMode::Overflowing),

        "u32overflowing_madd" => u32_ops::parse_u32madd(span_ops, op, U32OpMode::Overflowing),

        "u32checked_div" => u32_ops::parse_u32div(span_ops, op, U32OpMode::Checked),
        "u32unchecked_div" => u32_ops::parse_u32div(span_ops, op, U32OpMode::Unchecked),

        "u32checked_mod" => u32_ops::parse_u32mod(span_ops, op, U32OpMode::Checked),
        "u32unchecked_mod" => u32_ops::parse_u32mod(span_ops, op, U32OpMode::Unchecked),

        "u32checked_divmod" => u32_ops::parse_u32divmod(span_ops, op, U32OpMode::Checked),
        "u32unchecked_divmod" => u32_ops::parse_u32divmod(span_ops, op, U32OpMode::Unchecked),

        "u32checked_and" => u32_ops::parse_u32and(span_ops, op),
        "u32checked_or" => u32_ops::parse_u32or(span_ops, op),
        "u32checked_xor" => u32_ops::parse_u32xor(span_ops, op),
        "u32checked_not" => u32_ops::parse_u32not(span_ops, op),

        "u32checked_shr" => u32_ops::parse_u32shr(span_ops, op, U32OpMode::Checked),
        "u32unchecked_shr" => u32_ops::parse_u32shr(span_ops, op, U32OpMode::Unchecked),

        "u32checked_shl" => u32_ops::parse_u32shl(span_ops, op, U32OpMode::Checked),
        "u32unchecked_shl" => u32_ops::parse_u32shl(span_ops, op, U32OpMode::Unchecked),

        "u32checked_rotr" => u32_ops::parse_u32rotr(span_ops, op, U32OpMode::Checked),
        "u32unchecked_rotr" => u32_ops::parse_u32rotr(span_ops, op, U32OpMode::Unchecked),

        "u32checked_rotl" => u32_ops::parse_u32rotl(span_ops, op, U32OpMode::Checked),
        "u32unchecked_rotl" => u32_ops::parse_u32rotl(span_ops, op, U32OpMode::Unchecked),

        "u32checked_eq" => u32_ops::parse_u32eq(span_ops, op),
        "u32checked_neq" => u32_ops::parse_u32neq(span_ops, op),

        "u32checked_lt" => u32_ops::parse_u32lt(span_ops, op, U32OpMode::Checked),
        "u32unchecked_lt" => u32_ops::parse_u32lt(span_ops, op, U32OpMode::Unchecked),

        "u32checked_lte" => u32_ops::parse_u32lte(span_ops, op, U32OpMode::Checked),
        "u32unchecked_lte" => u32_ops::parse_u32lte(span_ops, op, U32OpMode::Unchecked),

        "u32checked_gt" => u32_ops::parse_u32gt(span_ops, op, U32OpMode::Checked),
        "u32unchecked_gt" => u32_ops::parse_u32gt(span_ops, op, U32OpMode::Unchecked),

        "u32checked_gte" => u32_ops::parse_u32gte(span_ops, op, U32OpMode::Checked),
        "u32unchecked_gte" => u32_ops::parse_u32gte(span_ops, op, U32OpMode::Unchecked),

        "u32checked_min" => u32_ops::parse_u32min(span_ops, op, U32OpMode::Checked),
        "u32unchecked_min" => u32_ops::parse_u32min(span_ops, op, U32OpMode::Unchecked),

        "u32checked_max" => u32_ops::parse_u32max(span_ops, op, U32OpMode::Checked),
        "u32unchecked_max" => u32_ops::parse_u32max(span_ops, op, U32OpMode::Unchecked),

        // ----- stack manipulation ---------------------------------------------------------------
        "drop" => stack_ops::parse_drop(span_ops, op),
        "dropw" => stack_ops::parse_dropw(span_ops, op),
        "padw" => stack_ops::parse_padw(span_ops, op),
        "dup" => stack_ops::parse_dup(span_ops, op),
        "dupw" => stack_ops::parse_dupw(span_ops, op),
        "swap" => stack_ops::parse_swap(span_ops, op),
        "swapw" => stack_ops::parse_swapw(span_ops, op),
        "swapdw" => stack_ops::parse_swapdw(span_ops, op),
        "movup" => stack_ops::parse_movup(span_ops, op),
        "movupw" => stack_ops::parse_movupw(span_ops, op),
        "movdn" => stack_ops::parse_movdn(span_ops, op),
        "movdnw" => stack_ops::parse_movdnw(span_ops, op),

        "cswap" => stack_ops::parse_cswap(span_ops, op),
        "cswapw" => stack_ops::parse_cswapw(span_ops, op),
        "cdrop" => stack_ops::parse_cdrop(span_ops, op),
        "cdropw" => stack_ops::parse_cdropw(span_ops, op),

        // ----- input / output operations --------------------------------------------------------
        "push" => io_ops::parse_push(span_ops, op, num_proc_locals),
        "pushw" => io_ops::parse_pushw(span_ops, op, num_proc_locals),
        "pop" => io_ops::parse_pop(span_ops, op, num_proc_locals),
        "popw" => io_ops::parse_popw(span_ops, op, num_proc_locals),
        "loadw" => io_ops::parse_loadw(span_ops, op, num_proc_locals),
        "storew" => io_ops::parse_storew(span_ops, op, num_proc_locals),
        "adv" => io_ops::parse_adv_inject(span_ops, op, decorators),

        // ----- cryptographic operations ---------------------------------------------------------
        "rphash" => crypto_ops::parse_rphash(span_ops, op),
        "rpperm" => crypto_ops::parse_rpperm(span_ops, op),

        "mtree" => crypto_ops::parse_mtree(span_ops, op, decorators),

        // ----- catch all ------------------------------------------------------------------------
        _ => return Err(AssemblyError::invalid_op(op)),
    }?;

    if in_debug_mode {
        let op_start = decorators[dec_len].0;
        // edit the number of cycles corresponding to the asmop decorator at an index
        if let Decorator::AsmOp(assembly_op) = &mut decorators[dec_len].1 {
            assembly_op.set_num_cycles((span_ops.len() - op_start) as u8)
        }
    }

    Ok(())
}

// HELPER FUNCTIONS
// ================================================================================================

/// Parses a single parameter into a valid field element.
fn parse_element_param(op: &Token, param_idx: usize) -> Result<Felt, AssemblyError> {
    // make sure that the parameter value is available
    if op.num_parts() <= param_idx {
        return Err(AssemblyError::missing_param(op));
    }
    let param_value = op.parts()[param_idx];

    if let Some(param_value) = param_value.strip_prefix("0x") {
        // parse hexadecimal number
        parse_hex_param(op, param_idx, param_value)
    } else {
        // parse decimal number
        parse_decimal_param(op, param_idx, param_value)
    }
}

/// Parses a decimal parameter value into valid a field element.
fn parse_decimal_param(
    op: &Token,
    param_idx: usize,
    param_str: &str,
) -> Result<Felt, AssemblyError> {
    match param_str.parse::<u64>() {
        Ok(value) => get_valid_felt(op, param_idx, value),
        Err(_) => Err(AssemblyError::invalid_param(op, param_idx)),
    }
}

/// Parses a hexadecimal parameter value into a valid field element.
fn parse_hex_param(op: &Token, param_idx: usize, param_str: &str) -> Result<Felt, AssemblyError> {
    match u64::from_str_radix(param_str, 16) {
        Ok(value) => get_valid_felt(op, param_idx, value),
        Err(_) => Err(AssemblyError::invalid_param(op, param_idx)),
    }
}

/// Checks that the u64 parameter value is a valid field element value and returns it as a field
/// element.
fn get_valid_felt(op: &Token, param_idx: usize, param: u64) -> Result<Felt, AssemblyError> {
    if param >= Felt::MODULUS {
        return Err(AssemblyError::invalid_param_with_reason(
            op,
            param_idx,
            format!("parameter value must be smaller than {}", Felt::MODULUS).as_str(),
        ));
    }

    Ok(Felt::new(param))
}

/// This is a helper function that parses the parameter at the specified op index as an integer and
/// ensures that it falls within the bounds specified by the caller.
///
/// # Errors
/// Returns an invalid param AssemblyError if:
/// - the parsing attempt fails.
/// - the parameter is outside the specified lower and upper bounds.
fn parse_u32_param(
    op: &Token,
    param_idx: usize,
    lower_bound: u32,
    upper_bound: u32,
) -> Result<u32, AssemblyError> {
    let param_value = op.parts()[param_idx];

    // attempt to parse the parameter value as an integer
    let result = match param_value.parse::<u32>() {
        Ok(i) => i,
        Err(_) => return Err(AssemblyError::invalid_param(op, param_idx)),
    };

    // check that the parameter is within the specified bounds
    if result < lower_bound || result > upper_bound {
        return Err(AssemblyError::invalid_param_with_reason(
            op,
            param_idx,
            format!(
                "parameter value must be greater than or equal to {} and less than or equal to {}",
                lower_bound, upper_bound
            )
            .as_str(),
        ));
    }

    Ok(result)
}

/// This is a helper function that appends a PUSH operation to the span block which puts the
/// provided value parameter onto the stack.
///
/// When the value is 0, PUSH operation is replaced with PAD. When the value is 1, PUSH operation
/// is replaced with PAD INCR because in most cases this will be more efficient than doing a PUSH.
fn push_value(span_ops: &mut Vec<Operation>, value: Felt) {
    if value == Felt::ZERO {
        span_ops.push(Operation::Pad);
    } else if value == Felt::ONE {
        span_ops.push(Operation::Pad);
        span_ops.push(Operation::Incr);
    } else {
        span_ops.push(Operation::Push(value));
    }
}

/// Validates an op Token against a provided instruction string and/or an expected number of
/// parameter inputs and returns an appropriate AssemblyError if the operation Token is invalid.
///
/// * To fully validate an operation, pass all of the following:
/// - the parsed operation Token
/// - a string describing a valid instruction, with variants separated by '|' and parameters
///   excluded.
/// - an integer or range for the number of allowed parameters
/// This will attempt to fully validate the operation, so a full-length instruction must be
/// described. For example, `popw.mem` accepts 0 or 1 inputs and can be validated by:
/// ```validate_operation!(op_token, "popw.mem", 0..1)```
///
/// * To validate only the operation parameters, specify @only_params before passing the same inputs
/// used for full validation (above). This will skip validating each part of the instruction.
/// For example, to validate only the parameters of `popw.mem` use:
/// ```validate_operation!(@only_params op_token, "popw.mem", 0..1)```
///
/// * To validate only the instruction portion of the operation, exclude the specification for the
/// number of parameters. This will only validate up to the number of parts in the provided
/// instruction string. For example, `pop.local` and `pop.mem` are the two valid instruction
/// variants for `pop`, so the first 2 parts of `pop` can be validated by:
/// ```validate_operation!(op_token, "pop.local|mem")```
/// or the first part can be validated by:
/// ```validate_operation!(op_token, "pop")```
#[macro_export]
macro_rules! validate_operation {
    // validate that the number of parameters is within the allowed range
    (@only_params $token:expr, $instr:literal, $min_params:literal..$max_params:expr ) => {
        let num_parts = $token.num_parts();
        let num_instr_parts = $instr.split(".").count();

        // token has too few parts to contain the required parameters
        if num_parts < num_instr_parts + $min_params {
            return Err(AssemblyError::missing_param($token));
        }
        // token has more than the maximum number of parts
        if num_parts > num_instr_parts + $max_params {
            return Err(AssemblyError::extra_param($token));
        }
    };
    // validate the exact number of parameters
    (@only_params $token:expr, $instr:literal, $num_params:literal) => {
        validate_operation!(@only_params $token, $instr, $num_params..$num_params);
    };

    // validate the instruction string and an optional parameter range
    ($token:expr, $instr:literal $(, $min_params:literal..$max_params:expr)?) => {
        // split the expected instruction into a vector of parts
        let instr_parts: Vec<Vec<&str>> = $instr
            .split(".")
            .map(|part| part.split("|").collect())
            .collect();

        let num_parts = $token.num_parts();
        let num_instr_parts = instr_parts.len();

        // token has too few parts to contain the full instruction
        if num_parts < num_instr_parts {
            return Err(AssemblyError::invalid_op($token));
        }

        // compare the parts to make sure they match
        for (part_variants, token_part) in instr_parts.iter().zip($token.parts()) {
            if !part_variants.contains(token_part) {
                return Err(AssemblyError::unexpected_token($token, $instr));
            }
        }

        $(
            // validate the parameter range, if provided
            validate_operation!(@only_params $token, $instr, $min_params..$max_params);
        )?
    };
    // validate the instruction string and an exact number of parameters
    ($token:expr, $instr:literal, $num_params:literal) => {
        validate_operation!($token, $instr, $num_params..$num_params);
    };
}
