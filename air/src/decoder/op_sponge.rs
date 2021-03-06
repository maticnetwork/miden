use super::{are_equal, BaseElement, EvaluationResult, FieldElement, UserOps, VmTransition};
use vm_core::op_sponge::{apply_inv_mds, apply_mds, apply_sbox, STATE_WIDTH};

// CONSTRAINT EVALUATOR
// ================================================================================================

pub fn enforce_hacc<E: FieldElement<BaseField = BaseElement>>(
    result: &mut [E],
    transition: &VmTransition<E>,
    ark: &[E],
    op_flag: E,
) {
    // determine current op_value
    let stack_top = transition.next().user_stack()[0];
    let push_flag = transition.hd_op_flags()[UserOps::Push.hd_index()];
    let op_value = stack_top * push_flag;

    // evaluate the first half of Rescue round
    let mut old_sponge = [E::ZERO; STATE_WIDTH];
    old_sponge.copy_from_slice(transition.current().op_sponge());
    for i in 0..STATE_WIDTH {
        old_sponge[i] += ark[i];
    }
    apply_sbox(&mut old_sponge);
    apply_mds(&mut old_sponge);

    // op_code injection
    old_sponge[0] += transition.current().op_code();
    old_sponge[1] += op_value;

    // evaluate inverse of the second half of Rescue round
    let mut new_sponge = [E::ZERO; STATE_WIDTH];
    new_sponge.copy_from_slice(transition.next().op_sponge());
    apply_inv_mds(&mut new_sponge);
    apply_sbox(&mut new_sponge);
    for i in 0..STATE_WIDTH {
        new_sponge[i] -= ark[STATE_WIDTH + i];
    }

    // add the constraints to the result
    for i in 0..STATE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(old_sponge[i], new_sponge[i]));
    }
}
