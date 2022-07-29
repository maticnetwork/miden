#![cfg_attr(not(feature = "std"), no_std)]

use air::{ProcessorAir, PublicInputs};
use core::fmt;
use vm_core::{utils::collections::Vec, MIN_STACK_DEPTH};
use winterfell::VerifierError;

// EXPORTS
// ================================================================================================

pub use assembly;
pub use vm_core::hasher::Digest;
pub use winterfell::StarkProof;

// VERIFIER
// ================================================================================================
/// Returns Ok(()) if the specified program was executed correctly against the specified inputs
/// and outputs.
///
/// Specifically, verifies that if a program with the specified `program_hash` is executed with the
/// provided `public_inputs` and some secret inputs, and the result is equal to the `outputs`.
///
/// # Errors
/// Returns an error if the provided proof does not prove a correct execution of the program.
pub fn verify(
    program_hash: Digest,
    stack_inputs: &[u64],
    stack_outputs: &[u64],
    proof: StarkProof,
) -> Result<(), VerificationError> {
    if stack_inputs.len() > MIN_STACK_DEPTH {
        return Err(VerificationError::TooManyInputValues(
            MIN_STACK_DEPTH,
            stack_inputs.len(),
        ));
    }

    // convert stack inputs to field elements
    let mut stack_input_felts = Vec::with_capacity(stack_inputs.len());
    for &input in stack_inputs.iter().rev() {
        stack_input_felts.push(
            input
                .try_into()
                .map_err(|_| VerificationError::InputNotFieldElement(input))?,
        );
    }

    if stack_outputs.len() > MIN_STACK_DEPTH {
        return Err(VerificationError::TooManyOutputValues(
            MIN_STACK_DEPTH,
            stack_outputs.len(),
        ));
    }

    // convert stack outputs to field elements
    let mut stack_output_felts = Vec::with_capacity(stack_outputs.len());
    for &output in stack_outputs.iter().rev() {
        stack_output_felts.push(
            output
                .try_into()
                .map_err(|_| VerificationError::OutputNotFieldElement(output))?,
        );
    }

    // build public inputs and try to verify the proof
    let pub_inputs = PublicInputs::new(program_hash, stack_input_felts, stack_output_felts);
    winterfell::verify::<ProcessorAir>(proof, pub_inputs).map_err(VerificationError::VerifierError)
}

// ERRORS
// ================================================================================================

/// TODO: add docs, implement Display
#[derive(Debug, PartialEq)]
pub enum VerificationError {
    VerifierError(VerifierError),
    InputNotFieldElement(u64),
    TooManyInputValues(usize, usize),
    OutputNotFieldElement(u64),
    TooManyOutputValues(usize, usize),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: implement friendly messages
        write!(f, "{:?}", self)
    }
}
