use crate::{
    chiplets::{ChipletsFrameExt, MemoryFrameExt},
    utils::are_equal,
};
use vm_core::{
    range::{P0_COL_IDX, P1_COL_IDX, Q_COL_IDX, S0_COL_IDX, S1_COL_IDX, T_COL_IDX, V_COL_IDX},
    utils::collections::Vec,
    ExtensionOf,
};
use winter_air::AuxTraceRandElements;

use super::{
    utils::{binary_not, is_binary},
    Assertion, EvaluationFrame, Felt, FieldElement, TransitionConstraintDegree,
};

// CONSTANTS
// ================================================================================================

// --- Main constraints ---------------------------------------------------------------------------

/// The number of boundary constraints required by the Range Checker
pub const NUM_ASSERTIONS: usize = 2;
/// The number of transition constraints required by the Range Checker.
pub const NUM_CONSTRAINTS: usize = 7;
/// The degrees of the range checker's constraints, in the order they'll be added to the the result
/// array when a transition is evaluated.
pub const CONSTRAINT_DEGREES: [usize; NUM_CONSTRAINTS] = [
    2, 2, 2, // Selector flags must be binary: t, s0, s1.
    3, // Constrain the row transitions in the 8-bit section of the table.
    2, // Transition from 8-bit to 16-bit section of range check table occurs at most once.
    3, 3, // Enforce values of column v before and after 8-bit to 16-bit transition.
];

// --- Auxiliary column constraints for multiset checks -------------------------------------------

/// The number of auxiliary assertions for multiset checks.
pub const NUM_AUX_ASSERTIONS: usize = 4;
/// The number of transition constraints required by multiset checks for the Range Checker.
pub const NUM_AUX_CONSTRAINTS: usize = 2;
/// The degrees of the Range Checker's auxiliary column constraints, used for multiset checks.
pub const AUX_CONSTRAINT_DEGREES: [usize; NUM_AUX_CONSTRAINTS] = [8, 8];

// BOUNDARY CONSTRAINTS
// ================================================================================================

// --- MAIN TRACE ---------------------------------------------------------------------------------

/// Returns the range checker's boundary assertions for the main trace at the first step.
pub fn get_assertions_first_step(result: &mut Vec<Assertion<Felt>>) {
    let step = 0;
    result.push(Assertion::single(V_COL_IDX, step, Felt::ZERO));
}

/// Returns the range checker's boundary assertions for the main trace at the last step.
pub fn get_assertions_last_step(result: &mut Vec<Assertion<Felt>>, step: usize) {
    result.push(Assertion::single(V_COL_IDX, step, Felt::new(65535)));
}

// --- AUXILIARY COLUMNS (FOR MULTISET CHECKS) ----------------------------------------------------

/// Returns the range checker's boundary assertions for auxiliary columns at the first step.
pub fn get_aux_assertions_first_step<E: FieldElement>(result: &mut Vec<Assertion<E>>) {
    let step = 0;
    result.push(Assertion::single(P0_COL_IDX, step, E::ONE));
    result.push(Assertion::single(P1_COL_IDX, step, E::ONE));
}

/// Returns the range checker's boundary assertions for auxiliary columns at the last step.
pub fn get_aux_assertions_last_step<E: FieldElement>(result: &mut Vec<Assertion<E>>, step: usize) {
    result.push(Assertion::single(P0_COL_IDX, step, E::ONE));
    result.push(Assertion::single(P1_COL_IDX, step, E::ONE));
}

// TRANSITION CONSTRAINTS
// ================================================================================================

// --- MAIN TRACE ---------------------------------------------------------------------------------

/// Builds the transition constraint degrees for the range checker.
pub fn get_transition_constraint_degrees() -> Vec<TransitionConstraintDegree> {
    CONSTRAINT_DEGREES
        .iter()
        .map(|&degree| TransitionConstraintDegree::new(degree))
        .collect()
}

/// Returns the number of transition constraints for the range checker.
pub fn get_transition_constraint_count() -> usize {
    NUM_CONSTRAINTS
}

/// Enforces constraints for the range checker.
pub fn enforce_constraints<E: FieldElement>(frame: &EvaluationFrame<E>, result: &mut [E]) {
    // Constrain the selector flags.
    let mut index = enforce_flags(frame, result);

    // Constrain the row transitions in the 8-bit section of the table.
    index += enforce_8bit(frame, &mut result[index..]);

    // Constrain the transition from 8-bit to 16-bit section of the table.
    enforce_16bit(frame, &mut result[index..]);
}

// --- AUXILIARY COLUMNS (FOR MULTISET CHECKS) ----------------------------------------------------

/// Returns the transition constraint degrees for the range checker's auxiliary columns, used for
/// multiset checks.
pub fn get_aux_transition_constraint_degrees() -> Vec<TransitionConstraintDegree> {
    AUX_CONSTRAINT_DEGREES
        .iter()
        .map(|&degree| TransitionConstraintDegree::new(degree))
        .collect()
}

/// Enforces constraints on the range checker's auxiliary columns.
pub fn enforce_aux_constraints<F, E>(
    main_frame: &EvaluationFrame<F>,
    aux_frame: &EvaluationFrame<E>,
    aux_rand_elements: &AuxTraceRandElements<E>,
    result: &mut [E],
) where
    F: FieldElement<BaseField = Felt>,
    E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
{
    // Get the first random element for this segment.
    let alpha = aux_rand_elements.get_segment_elements(0)[0];

    // Enforce p0.
    let index = enforce_running_product_p0(main_frame, aux_frame, alpha, result);

    // Enforce p1.
    enforce_running_product_p1(main_frame, aux_frame, alpha, &mut result[index..]);
}

// TRANSITION CONSTRAINT HELPERS
// ================================================================================================

// --- MAIN TRACE ---------------------------------------------------------------------------------

/// Constrain the selector flags to binary values.
fn enforce_flags<E: FieldElement>(frame: &EvaluationFrame<E>, result: &mut [E]) -> usize {
    let constraint_count = 3;

    result[0] = is_binary(frame.t());
    result[1] = is_binary(frame.s0());
    result[2] = is_binary(frame.s1());

    constraint_count
}

/// Constrain the row transitions in the 8-bit section of the table.
fn enforce_8bit<E: FieldElement>(frame: &EvaluationFrame<E>, result: &mut [E]) -> usize {
    let constraint_count = 1;
    let v_change = frame.change(V_COL_IDX);

    result[0] = binary_not(frame.t_next()) * (v_change) * (v_change - E::ONE);

    constraint_count
}

/// Constrain the transition from 8-bit to 16-bit section of the table.
fn enforce_16bit<E: FieldElement>(frame: &EvaluationFrame<E>, result: &mut [E]) -> usize {
    let constraint_count = 3;

    // Values in column t can "flip" from 0 to 1 only once.
    result[0] = frame.t() * binary_not(frame.t_next());

    // When column t "flips", column v must equal 255.
    result[1] = frame.flip_to_16bit_flag() * (frame.v() - E::from(255_u8));

    // When column t "flips", the next value column v must be reset to 0.
    result[2] = frame.flip_to_16bit_flag() * frame.v_next();

    constraint_count
}

// --- AUXILIARY COLUMNS (FOR MULTISET CHECKS) ----------------------------------------------------

/// Ensures that the running product auxiliary column `p0` is correctly built up during the 8-bit
/// section of the range check table and then correctly reduced in the 16-bit section of the table.
///
/// In the 8-bit section, when `t=0` the value of `z` is included in the running product at each
/// step, and the constraint reduces to p0' = p0 * z.
/// In the 16-bit section, when `t=1`, the running product is reduced by the difference between the
/// current and next value, and the constraint reduces to p0' * (alpha + v' - v) = p0.
fn enforce_running_product_p0<E, F>(
    main_frame: &EvaluationFrame<F>,
    aux_frame: &EvaluationFrame<E>,
    alpha: E,
    result: &mut [E],
) -> usize
where
    F: FieldElement<BaseField = Felt>,
    E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
{
    let mut constraint_offset = 0;

    let z = get_z(main_frame, alpha);
    let t = main_frame.t().into();
    let p0_term = aux_frame.p0() * (z - z * t + t);
    let p0_next_term = aux_frame.p0_next()
        * ((alpha + main_frame.v_next().into() - main_frame.v().into()) * t - t + E::ONE);
    result[constraint_offset] = p0_next_term - p0_term;
    constraint_offset += 1;

    constraint_offset
}

/// Ensures that the 16-bit running product is computed correctly in the column `p1`. It enforces
/// that the value only changes during the 16-bit section of the table, where the value of `z` is
/// included at each step, ensuring that the values in the 16-bit section are multiplied into `p1`
/// 0, 1, 2, or 4 times, according to the selector flags.
fn enforce_running_product_p1<E, F>(
    main_frame: &EvaluationFrame<F>,
    aux_frame: &EvaluationFrame<E>,
    alpha: E,
    result: &mut [E],
) where
    F: FieldElement<BaseField = Felt>,
    E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
{
    // The running product column must enforce that the next step has the values from the range
    // checker multiplied in (z) and the values from the stack (q) and the memory divided out. This
    // is enforced by ensuring that p1_next multiplied by the stack and memory lookups at this step
    // is equal to the combination of p1 and the range checker's values for this step.
    let lookups = aux_frame.q() * get_memory_lookups(main_frame, alpha);
    let t: E = main_frame.t().into();
    let range_checks = get_z(main_frame, alpha) * t - t + E::ONE;

    result[0] = are_equal(aux_frame.p1_next() * lookups, aux_frame.p1() * range_checks);
}

/// The value to be included in the running product column for memory lookups at this row. These are
/// only included for steps in the memory section of the trace (when the memory_flag is one).
fn get_memory_lookups<E, F>(main_frame: &EvaluationFrame<F>, alpha: E) -> E
where
    F: FieldElement<BaseField = Felt>,
    E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
{
    let memory_flag: E = main_frame.chiplets_memory_flag().into();
    let d0: E = main_frame.memory_d0().into();
    let d1: E = main_frame.memory_d1().into();

    E::ONE + memory_flag * ((d0 + alpha) * (d1 + alpha) - E::ONE)
}

/// Returns the value `z` which is included in the running product columns at each step. `z` causes
/// the row's value to be included 0, 1, 2, or 4 times, according to the row's selector flags row.
fn get_z<E, F>(main_frame: &EvaluationFrame<F>, alpha: E) -> E
where
    F: FieldElement<BaseField = Felt>,
    E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
{
    // Get the selectors and the value from the main frame.
    let s0: E = main_frame.s0().into();
    let s1: E = main_frame.s1().into();
    let v: E = main_frame.v().into();

    // Define the flags.
    let f0: E = binary_not(s0) * binary_not(s1);
    let f1: E = s0 * binary_not(s1);
    let f2: E = binary_not(s0) * s1;
    let f3: E = s0 * s1;

    // Compute z.
    let v_alpha = v + alpha;
    let v_alpha2 = v_alpha.square();
    let v_alpha4 = v_alpha2.square();
    f3 * v_alpha4 + f2 * v_alpha2 + f1 * v_alpha + f0
}

// RANGE CHECKER FRAME EXTENSION TRAIT
// ================================================================================================

/// Trait to allow easy access to column values and intermediate variables used in constraint
/// calculations for the Range Checker.
trait EvaluationFrameExt<E: FieldElement> {
    // --- Column accessors -----------------------------------------------------------------------

    /// The current value in column T.
    fn t(&self) -> E;
    /// The next value in column T.
    fn t_next(&self) -> E;
    /// The current value in column s0.
    fn s0(&self) -> E;
    /// The current value in column s1.
    fn s1(&self) -> E;
    /// The current value in column V.
    fn v(&self) -> E;
    /// The next value in column V.
    fn v_next(&self) -> E;
    /// The current value in auxiliary column p0.
    fn p0(&self) -> E;
    /// The next value in auxiliary column p0.
    fn p0_next(&self) -> E;
    /// The current value in auxiliary column p1.
    fn p1(&self) -> E;

    /// The next value in auxiliary column p1.
    fn p1_next(&self) -> E;

    /// The current value in auxiliary column q.
    fn q(&self) -> E;

    // --- Intermediate variables & helpers -------------------------------------------------------

    /// The change between the current value in the specified column and the next value, calculated
    /// as `next - current`.
    fn change(&self, column: usize) -> E;

    // --- Flags ----------------------------------------------------------------------------------

    /// A flag set to 1 when column t changes for 0 to 1 indicating the transition from the 8-bit to
    /// 16-bit sections of the range checker table.
    fn flip_to_16bit_flag(&self) -> E;
}

impl<E: FieldElement> EvaluationFrameExt<E> for &EvaluationFrame<E> {
    // --- Column accessors -----------------------------------------------------------------------

    #[inline(always)]
    fn t(&self) -> E {
        self.current()[T_COL_IDX]
    }

    #[inline(always)]
    fn t_next(&self) -> E {
        self.next()[T_COL_IDX]
    }

    #[inline(always)]
    fn s0(&self) -> E {
        self.current()[S0_COL_IDX]
    }

    #[inline(always)]
    fn s1(&self) -> E {
        self.current()[S1_COL_IDX]
    }

    #[inline(always)]
    fn v(&self) -> E {
        self.current()[V_COL_IDX]
    }

    #[inline(always)]
    fn v_next(&self) -> E {
        self.next()[V_COL_IDX]
    }

    #[inline(always)]
    fn p0(&self) -> E {
        self.current()[P0_COL_IDX]
    }

    #[inline(always)]
    fn p0_next(&self) -> E {
        self.next()[P0_COL_IDX]
    }

    #[inline(always)]
    fn p1(&self) -> E {
        self.current()[P1_COL_IDX]
    }

    #[inline(always)]
    fn p1_next(&self) -> E {
        self.next()[P1_COL_IDX]
    }

    #[inline(always)]
    fn q(&self) -> E {
        self.current()[Q_COL_IDX]
    }

    // --- Intermediate variables & helpers -------------------------------------------------------

    #[inline(always)]
    fn change(&self, column: usize) -> E {
        self.next()[column] - self.current()[column]
    }

    // --- Flags ----------------------------------------------------------------------------------

    #[inline(always)]
    fn flip_to_16bit_flag(&self) -> E {
        binary_not(self.t()) * self.t_next()
    }
}
