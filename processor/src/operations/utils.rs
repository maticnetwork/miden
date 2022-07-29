use super::{ExecutionError, Felt, FieldElement};

/// TODO: add docs
#[inline(always)]
pub fn assert_binary(value: Felt) -> Result<Felt, ExecutionError> {
    if value != Felt::ZERO && value != Felt::ONE {
        Err(ExecutionError::NotBinaryValue(value))
    } else {
        Ok(value)
    }
}
