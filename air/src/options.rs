use core::ops::Deref;
use winter_air::{FieldExtension, HashFunction, ProofOptions as WinterProofOptions};

/// TODO: add docs
#[derive(Clone)]
pub struct ProofOptions(WinterProofOptions);

impl ProofOptions {
    pub fn new(
        num_queries: usize,
        blowup_factor: usize,
        grinding_factor: u32,
        hash_fn: HashFunction,
        field_extension: FieldExtension,
        fri_folding_factor: usize,
        fri_max_remainder_size: usize,
    ) -> Self {
        Self(WinterProofOptions::new(
            num_queries,
            blowup_factor,
            grinding_factor,
            hash_fn,
            field_extension,
            fri_folding_factor,
            fri_max_remainder_size,
        ))
    }

    pub fn with_96_bit_security() -> Self {
        Self(WinterProofOptions::new(
            27,
            8,
            16,
            HashFunction::Blake3_192,
            FieldExtension::Quadratic,
            8,
            256,
        ))
    }

    pub fn with_128_bit_security() -> Self {
        Self(WinterProofOptions::new(
            27,
            16,
            21,
            HashFunction::Blake3_256,
            FieldExtension::Cubic,
            8,
            256,
        ))
    }

    pub fn into_inner(self) -> WinterProofOptions {
        self.0
    }
}

impl Default for ProofOptions {
    fn default() -> Self {
        Self::with_96_bit_security()
    }
}

impl Deref for ProofOptions {
    type Target = WinterProofOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
