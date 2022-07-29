use super::{
    ExecutionError, Felt, FieldElement, Join, Loop, OpBatch, Operation, Process, Span, Split,
    StarkField, Vec, Word, MIN_TRACE_LEN, ONE, OP_BATCH_SIZE, ZERO,
};
use vm_core::{
    code_blocks::get_span_op_group_count,
    decoder::{
        NUM_HASHER_COLUMNS, NUM_OP_BATCH_FLAGS, NUM_OP_BITS, OP_BATCH_1_GROUPS, OP_BATCH_2_GROUPS,
        OP_BATCH_4_GROUPS, OP_BATCH_8_GROUPS,
    },
    hasher::DIGEST_LEN,
    AssemblyOp,
};

mod trace;
use trace::DecoderTrace;

mod aux_hints;
pub use aux_hints::{
    AuxTraceHints, BlockHashTableRow, BlockStackTableRow, BlockTableUpdate, OpGroupTableRow,
    OpGroupTableUpdate,
};

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const HASH_CYCLE_LEN: Felt = Felt::new(vm_core::hasher::HASH_CYCLE_LEN as u64);

// DECODER PROCESS EXTENSION
// ================================================================================================

impl Process {
    // JOIN BLOCK
    // --------------------------------------------------------------------------------------------

    /// Starts decoding of a JOIN block.
    pub(super) fn start_join_block(&mut self, block: &Join) -> Result<(), ExecutionError> {
        // use the hasher to compute the hash of the JOIN block; the row address returned by the
        // hasher is used as the ID of the block; the result of the hash is expected to be in
        // row addr + 7.
        let child1_hash = block.first().hash().into();
        let child2_hash = block.second().hash().into();
        let (addr, _result) = self.chiplets.merge(child1_hash, child2_hash);

        // make sure the result computed by the hasher is the same as the expected block hash
        debug_assert_eq!(block.hash(), _result.into());

        // start decoding the JOIN block; this appends a row with JOIN operation to the decoder
        // trace. when JOIN operation is executed, the rest of the VM state does not change
        self.decoder.start_join(child1_hash, child2_hash, addr);
        self.execute_op(Operation::Noop)
    }

    ///  Ends decoding of a JOIN block.
    pub(super) fn end_join_block(&mut self, block: &Join) -> Result<(), ExecutionError> {
        // this appends a row with END operation to the decoder trace. when END operation is
        // executed the rest of the VM state does not change
        self.decoder.end_control_block(block.hash().into());
        self.execute_op(Operation::Noop)
    }

    // SPLIT BLOCK
    // --------------------------------------------------------------------------------------------

    /// Starts decoding a SPLIT block. This also pops the value from the top of the stack and
    /// returns it.
    pub(super) fn start_split_block(&mut self, block: &Split) -> Result<Felt, ExecutionError> {
        let condition = self.stack.peek();

        // use the hasher to compute the hash of the SPLIT block; the row address returned by the
        // hasher is used as the ID of the block; the result of the hash is expected to be in
        // row addr + 7.
        let child1_hash = block.on_true().hash().into();
        let child2_hash = block.on_false().hash().into();
        let (addr, _result) = self.chiplets.merge(child1_hash, child2_hash);

        // make sure the result computed by the hasher is the same as the expected block hash
        debug_assert_eq!(block.hash(), _result.into());

        // start decoding the SPLIT block. this appends a row with SPLIT operation to the decoder
        // trace. we also pop the value off the top of the stack and return it.
        self.decoder
            .start_split(child1_hash, child2_hash, addr, condition);
        self.execute_op(Operation::Drop)?;
        Ok(condition)
    }

    /// Ends decoding of a SPLIT block.
    pub(super) fn end_split_block(&mut self, block: &Split) -> Result<(), ExecutionError> {
        // this appends a row with END operation to the decoder trace. when END operation is
        // executed the rest of the VM state does not change
        self.decoder.end_control_block(block.hash().into());
        self.execute_op(Operation::Noop)
    }

    // LOOP BLOCK
    // --------------------------------------------------------------------------------------------

    /// Starts decoding a LOOP block. This also pops the value from the top of the stack and
    /// returns it.
    pub(super) fn start_loop_block(&mut self, block: &Loop) -> Result<Felt, ExecutionError> {
        let condition = self.stack.peek();

        // use the hasher to compute the hash of the LOOP block; for LOOP block there is no
        // second child so we set the second hash to ZEROs; the row address returned by the
        // hasher is used as the ID of the block; the result of the hash is expected to be in
        // row addr + 7.
        let body_hash = block.body().hash().into();
        let (addr, _result) = self.chiplets.merge(body_hash, [ZERO; 4]);

        // make sure the result computed by the hasher is the same as the expected block hash
        debug_assert_eq!(block.hash(), _result.into());

        // start decoding the LOOP block; this appends a row with LOOP operation to the decoder
        // trace, but if the value on the top of the stack is not ONE, the block is not marked
        // as the loop block, and the hash of the body will not be added to the block hash table.
        // basically, if the top of the stack is ZERO, a LOOP operation should be immediately
        // followed by an END operation.
        self.decoder.start_loop(body_hash, addr, condition);
        self.execute_op(Operation::Drop)?;
        Ok(condition)
    }

    /// Ends decoding of a LOOP block. If pop_stack is set to true, this also removes the
    /// value at the top of the stack.
    pub(super) fn end_loop_block(
        &mut self,
        block: &Loop,
        pop_stack: bool,
    ) -> Result<(), ExecutionError> {
        // this appends a row with END operation to the decoder trace.
        self.decoder.end_control_block(block.hash().into());

        // if we are exiting a loop, we also need to pop the top value off the stack (and this
        // value must be ZERO - otherwise, we should have stayed in the loop). but, if we never
        // entered the loop in the first place, the stack would have been popped when the LOOP
        // operation was executed.
        if pop_stack {
            #[cfg(debug_assertions)]
            {
                let condition = self.stack.peek();
                debug_assert_eq!(ZERO, condition);
            }

            self.execute_op(Operation::Drop)
        } else {
            self.execute_op(Operation::Noop)
        }
    }

    // SPAN BLOCK
    // --------------------------------------------------------------------------------------------

    /// Starts decoding a SPAN block.
    pub(super) fn start_span_block(&mut self, block: &Span) -> Result<(), ExecutionError> {
        // use the hasher to compute the hash of the SPAN block; the row address returned by the
        // hasher is used as the ID of the block; hash of a SPAN block is computed by sequentially
        // hashing operation batches. Thus, the result of the hash is expected to be in row
        // addr + (num_batches * 8) - 1.
        let op_batches = block.op_batches();
        let num_op_groups = get_span_op_group_count(op_batches);
        let (addr, _result) = self.chiplets.hash_span_block(op_batches, num_op_groups);

        // make sure the result computed by the hasher is the same as the expected block hash
        debug_assert_eq!(block.hash(), _result.into());

        // start decoding the first operation batch; this also appends a row with SPAN operation
        // to the decoder trace. we also need the total number of operation groups so that we can
        // set the value of the group_count register at the beginning of the SPAN.
        self.decoder
            .start_span(&op_batches[0], Felt::new(num_op_groups as u64), addr);
        self.execute_op(Operation::Noop)
    }

    /// Ends decoding a SPAN block.
    pub(super) fn end_span_block(&mut self, block: &Span) -> Result<(), ExecutionError> {
        // this appends a row with END operation to the decoder trace. when END operation is
        // executed the rest of the VM state does not change
        self.decoder.end_span(block.hash().into());
        self.execute_op(Operation::Noop)
    }
}

// DECODER
// ================================================================================================

/// Program decoder for the VM.
///
/// This component is responsible for decoding operations executed on the VM, computing the hash
/// of the executed program, as well as building an execution trace for these computations.
///
/// ## Execution trace
/// Decoder execution trace currently consists of 22 columns as illustrated below (this will
/// be increased to 23 columns in the future):
///
///  addr b0 b1 b2 b3 b4 b5 b6 h0 h1 h2 h3 h4 h5 h6 h7 in_span g_count op_idx c0 c1 c2
/// ├────┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴───────┴───────┴──────┴──┴──┴──┤
///
/// In the above, the meaning of the columns is as follows:
/// * addr column contains address of the hasher for the current block (row index from the
///   auxiliary hashing table). It also serves the role of unique block identifiers. This is
///   convenient, because hasher addresses are guaranteed to be unique.
/// * op_bits columns b0 through b6 are used to encode an operation to be executed by the VM.
///   Each of these columns contains a single binary value, which together form a single opcode.
/// * Hasher state columns h0 through h7. These are multi purpose columns used as follows:
///   - When starting decoding of a new code block (e.g., via JOIN, SPLIT, LOOP, SPAN operations)
///    these columns are used for providing inputs for the current block's hash computations.
///   - When finishing decoding of a code block (i.e., via END operation), these columns are
///     used to record the result of the hash computation.
///   - Inside a SPAN block, the first two columns are used to keep track of un-executed
///     operations in the current operation group, as well as the address of the parent code
///     block. The remaining 6 columns are unused by the decoder and, thus, can be used by the
///     VM as helper columns.
/// * in_span column is a binary flag set to ONE when we are inside a SPAN block, and to ZERO
///   otherwise.
/// * Operation group count column is used to keep track of the number of un-executed operation
///   groups in the current SPAN block.
/// * Operation index column is used to keep track of the indexes of the currently executing
///   operations within an operation group. Values in this column could be between 0 and 8
///   (both inclusive) as there could be at most 9 operations in an operation group.
/// * Operation batch flag columns c0, c1, c2 which indicate how many operation groups are in
///   a given operation batch. These flags are set only for SPAN or RESPAN operations, and are
///   set to ZEROs otherwise.
///
/// In addition to the execution trace, the decoder also contains the following:
/// - A set of hints used in construction of decoder-related columns in auxiliary trace segment.
/// - An instance of [DebugInfo] which is only populated in debug mode. This debug_info instance
///   includes operations executed by the VM and AsmOp decorators. AsmOp decorators are popoulated
///   only when both the processor and assembler are in debug mode.
pub struct Decoder {
    block_stack: BlockStack,
    span_context: Option<SpanContext>,
    trace: DecoderTrace,
    aux_hints: AuxTraceHints,
    debug_info: DebugInfo,
}

impl Decoder {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns an empty instance of [Decoder].
    pub fn new(in_debug_mode: bool) -> Self {
        Self {
            block_stack: BlockStack::new(),
            span_context: None,
            trace: DecoderTrace::new(),
            aux_hints: AuxTraceHints::new(),
            debug_info: DebugInfo::new(in_debug_mode),
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns execution trace length for this decoder.
    pub fn trace_len(&self) -> usize {
        self.trace.trace_len()
    }

    /// Hash of the program decoded by this decoder.
    ///
    /// Hash of the program is taken from the last row of first 4 registers of the hasher section
    /// of the decoder trace (i.e., columns 8 - 12).
    pub fn program_hash(&self) -> [Felt; DIGEST_LEN] {
        self.trace.program_hash()
    }

    pub fn debug_info(&self) -> &DebugInfo {
        debug_assert!(self.in_debug_mode());
        &self.debug_info
    }

    /// Returns whether this decoder instance is instantiated in debug mode.
    pub fn in_debug_mode(&self) -> bool {
        self.debug_info.in_debug_mode()
    }

    // CONTROL BLOCKS
    // --------------------------------------------------------------------------------------------

    /// Starts decoding of a JOIN block.
    ///
    /// This pushes a block with ID=addr onto the block stack and appends execution of a JOIN
    /// operation to the trace.
    pub fn start_join(&mut self, child1_hash: Word, child2_hash: Word, addr: Felt) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // append a JOIN row to the execution trace
        let parent_addr = self.block_stack.push(addr, BlockType::Join(false));
        self.trace
            .append_block_start(parent_addr, Operation::Join, child1_hash, child2_hash);

        // mark this cycle as the cycle at which a new JOIN block began execution (this affects
        // block stack and block hash tables). Both children of the JOIN block are expected to
        // be executed, and thus we record both of their hashes.
        self.aux_hints.block_started(
            clk,
            self.block_stack.peek(),
            Some(child1_hash),
            Some(child2_hash),
        );

        self.debug_info.append_operation(Operation::Join);
    }

    /// Starts decoding of a SPLIT block.
    ///
    /// This pushes a block with ID=addr onto the block stack and appends execution of a SPLIT
    /// operation to the trace.
    pub fn start_split(
        &mut self,
        child1_hash: Word,
        child2_hash: Word,
        addr: Felt,
        stack_top: Felt,
    ) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // append a SPLIT row to the execution trace
        let parent_addr = self.block_stack.push(addr, BlockType::Split);
        self.trace
            .append_block_start(parent_addr, Operation::Split, child1_hash, child2_hash);

        // mark this cycle as the cycle at which a SPLIT block began execution (this affects block
        // stack and block hash tables). Only one child of the SPLIT block is expected to be
        // executed, and thus, we record the hash only for that child.
        let taken_branch_hash = if stack_top == ONE {
            child1_hash
        } else {
            child2_hash
        };
        self.aux_hints
            .block_started(clk, self.block_stack.peek(), Some(taken_branch_hash), None);

        self.debug_info.append_operation(Operation::Split);
    }

    /// Starts decoding of a LOOP block.
    ///
    /// This pushes a block with ID=addr onto the block stack and appends execution of a LOOP
    /// operation to the trace. A block is marked as a loop block only if is_loop = ONE.
    pub fn start_loop(&mut self, loop_body_hash: Word, addr: Felt, stack_top: Felt) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // append a LOOP row to the execution trace
        let enter_loop = stack_top == ONE;
        let parent_addr = self.block_stack.push(addr, BlockType::Loop(enter_loop));
        self.trace
            .append_block_start(parent_addr, Operation::Loop, loop_body_hash, [ZERO; 4]);

        // mark this cycle as the cycle at which a new LOOP block has started (this may affect
        // block hash table). A loop block has a single child only if the body of the loop is
        // executed at least once.
        let executed_loop_body = if enter_loop {
            Some(loop_body_hash)
        } else {
            None
        };
        self.aux_hints
            .block_started(clk, self.block_stack.peek(), executed_loop_body, None);

        self.debug_info.append_operation(Operation::Loop);
    }

    /// Starts decoding another iteration of a loop.
    ///
    /// This appends an execution of a REPEAT operation to the trace.
    pub fn repeat(&mut self) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // append a REPEAT row to the execution trace
        let block_info = self.block_stack.peek();
        debug_assert_eq!(ONE, block_info.is_entered_loop());
        self.trace.append_loop_repeat(block_info.addr);

        // mark this cycle as the cycle at which a new iteration of a loop started (this affects
        // block hash table)
        self.aux_hints.loop_repeat_started(clk);

        self.debug_info.append_operation(Operation::Repeat);
    }

    /// Ends decoding of a control block (i.e., a non-SPAN block).
    ///
    /// This appends an execution of an END operation to the trace. The top block on the block
    /// stack is also popped.
    pub fn end_control_block(&mut self, block_hash: Word) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // add an END row to the trace
        let block_info = self.block_stack.pop();
        self.trace.append_block_end(
            block_info.addr,
            block_hash,
            block_info.is_loop_body(),
            block_info.is_entered_loop(),
        );

        // mark this cycle as the cycle at which block execution has ended
        self.aux_hints.block_ended(clk, block_info.is_first_child);

        self.debug_info.append_operation(Operation::End);
    }

    // SPAN BLOCK
    // --------------------------------------------------------------------------------------------

    /// Starts decoding of a SPAN block defined by the specified operation batches.
    pub fn start_span(&mut self, first_op_batch: &OpBatch, num_op_groups: Felt, addr: Felt) {
        debug_assert!(self.span_context.is_none(), "already in span");
        let parent_addr = self.block_stack.push(addr, BlockType::Span);

        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // add a SPAN row to the trace
        self.trace
            .append_span_start(parent_addr, first_op_batch.groups(), num_op_groups);

        // after SPAN operation is executed, we decrement the number of remaining groups by ONE
        // because executing SPAN consumes the first group of the batch.
        self.span_context = Some(SpanContext {
            num_groups_left: num_op_groups - ONE,
            group_ops_left: first_op_batch.groups()[0],
        });

        // mark the current cycle as a cycle at which an operation batch may have been inserted
        // into the op_group table
        self.aux_hints.insert_op_batch(clk, num_op_groups);

        // mark the current cycle as the cycle at which a SPAN block has started; SPAN block has
        // no children
        self.aux_hints
            .block_started(clk, self.block_stack.peek(), None, None);

        self.debug_info.append_operation(Operation::Span);
    }

    /// Starts decoding of the next operation batch in the current SPAN.
    pub fn respan(&mut self, op_batch: &OpBatch) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // add RESPAN row to the trace
        self.trace.append_respan(op_batch.groups());

        // we also need to increment block address by 8 because hashing every additional operation
        // batch requires 8 rows of the hasher trace.
        let block_info = self.block_stack.peek_mut();
        block_info.addr += HASH_CYCLE_LEN;

        let ctx = self.span_context.as_mut().expect("not in span");

        // mark the current cycle as a cycle at which an operation batch may have been inserted
        // into the op_group table
        self.aux_hints.insert_op_batch(clk, ctx.num_groups_left);

        // mark the current cycle as a cycle at which the ID of the span block was changed (this
        // causes an update in the block stack table)
        self.aux_hints.span_extended(clk, block_info);

        // after RESPAN operation is executed, we decrement the number of remaining groups by ONE
        // because executing RESPAN consumes the first group of the batch
        ctx.num_groups_left -= ONE;
        ctx.group_ops_left = op_batch.groups()[0];

        self.debug_info.append_operation(Operation::Respan);
    }

    /// Starts decoding a new operation group.
    pub fn start_op_group(&mut self, op_group: Felt) {
        let clk = self.trace_len();
        let ctx = self.span_context.as_mut().expect("not in span");

        // mark the cycle of the last operation as a cycle at which an operation group was
        // removed from the op_group table. decoding of the removed operation will begin
        // at the current cycle.
        let group_pos = ctx.num_groups_left;
        let batch_id = self.block_stack.peek().addr;
        self.aux_hints
            .remove_op_group(clk - 1, batch_id, group_pos, op_group);

        // reset the current group value and decrement the number of left groups by ONE
        debug_assert_eq!(
            ZERO, ctx.group_ops_left,
            "not all ops executed in current group"
        );
        ctx.group_ops_left = op_group;
        ctx.num_groups_left -= ONE;
    }

    /// Decodes a user operation (i.e., not a control flow operation).
    pub fn execute_user_op(&mut self, op: Operation, op_idx: usize) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        let block = self.block_stack.peek();
        let ctx = self.span_context.as_mut().expect("not in span");

        // update operations left to be executed in the group
        ctx.group_ops_left = remove_opcode_from_group(ctx.group_ops_left, op);

        // append the row for the operation to the trace
        self.trace.append_user_op(
            op,
            block.addr,
            block.parent_addr,
            ctx.num_groups_left,
            ctx.group_ops_left,
            Felt::from(op_idx as u32),
        );

        // if the operation carries an immediate value, decrement the number of  operation
        // groups left to decode. this number will be inserted into the trace in the next row.
        // we also mark the current clock cycle as a cycle at which the immediate value was
        // removed from the op_group table.
        if let Some(imm_value) = op.imm_value() {
            let group_pos = ctx.num_groups_left;
            self.aux_hints
                .remove_op_group(clk, block.addr, group_pos, imm_value);

            ctx.num_groups_left -= ONE;
        }

        self.debug_info.append_operation(op);
    }

    /// Sets the helper registers in the trace to the user-provided helper values. This is expected
    /// to be called during the execution of a user operation.
    ///
    /// TODO: it might be better to get the operation information from the decoder trace, rather
    /// than passing it in as a parameter.
    pub fn set_user_op_helpers(&mut self, op: Operation, values: &[Felt]) {
        debug_assert!(!op.is_control_op(), "op is a control operation");
        self.trace.set_user_op_helpers(values);
    }

    /// Ends decoding of a SPAN block.
    pub fn end_span(&mut self, block_hash: Word) {
        // get the current clock cycle here (before the trace table is updated)
        let clk = self.trace_len();

        // remove the block from the stack of executing blocks and add an END row to the
        // execution trace
        let block_info = self.block_stack.pop();
        self.trace
            .append_span_end(block_hash, block_info.is_loop_body());
        self.span_context = None;

        // mark this cycle as the cycle at which block execution has ended
        self.aux_hints.block_ended(clk, block_info.is_first_child);

        self.debug_info.append_operation(Operation::End);
    }

    // TRACE GENERATIONS
    // --------------------------------------------------------------------------------------------

    /// Returns an array of columns containing an execution trace of this decoder together with
    /// hints to be used in construction of decoder-related auxiliary trace segment columns.
    ///
    /// Trace columns are extended to match the specified trace length.
    pub fn into_trace(mut self, trace_len: usize, num_rand_rows: usize) -> super::DecoderTrace {
        // once we know the hash of the program, we update the auxiliary trace hints so that the
        // block hash table could be initialized properly
        self.aux_hints.set_program_hash(self.program_hash());

        let trace = self
            .trace
            .into_vec(trace_len, num_rand_rows)
            .try_into()
            .expect("failed to convert vector to array");

        super::DecoderTrace {
            trace,
            aux_trace_hints: self.aux_hints,
        }
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Appends an asmop decorator at the specified clock cycle to the asmop list in debug mode.
    pub fn append_asmop(&mut self, clk: usize, asmop: AssemblyOp) {
        self.debug_info.append_asmop(clk, asmop);
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------

    /// Adds a row of zeros to the decoder trace for testing purposes.
    #[cfg(test)]
    pub fn add_dummy_trace_row(&mut self) {
        self.trace.add_dummy_row();
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new(false)
    }
}

// BLOCK STACK
// ================================================================================================

/// Keeps track of code blocks which are currently being executed by the VM.
struct BlockStack {
    blocks: Vec<BlockInfo>,
}

impl BlockStack {
    /// Returns an empty [BlockStack].
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// Pushes a new code block onto the block stack and returns the address of the block's parent.
    ///
    /// The block is identified by its address, and we also need to know what type of a block this
    /// is. Other information (i.e., the block's parent, whether the block is a body of
    /// a loop or a first child of a JOIN block) is determined from the information already on the
    /// stack.
    pub fn push(&mut self, addr: Felt, block_type: BlockType) -> Felt {
        let (parent_addr, is_loop_body, is_first_child) = match self.blocks.last() {
            Some(parent) => match parent.block_type {
                // if the parent is a LOOP block, this block must be a loop body
                BlockType::Loop(loop_entered) => {
                    debug_assert!(loop_entered, "parent is un-entered loop");
                    (parent.addr, true, false)
                }
                // if the parent is a JOIN block, figure out if this block is the first or the
                // second child
                BlockType::Join(first_child_executed) => {
                    (parent.addr, false, !first_child_executed)
                }
                _ => (parent.addr, false, false),
            },
            // if the block has no parent, it is neither a body of a loop nor the first child of
            // a JOIN block; also, we set the parent address to ZERO.
            None => (ZERO, false, false),
        };

        self.blocks.push(BlockInfo {
            addr,
            block_type,
            parent_addr,
            is_loop_body,
            is_first_child,
        });
        parent_addr
    }

    /// Removes a block from the top of the stack and returns it.
    pub fn pop(&mut self) -> BlockInfo {
        let block = self.blocks.pop().expect("block stack is empty");
        // if the parent block is a JOIN block (i.e., we just finished executing a child of a JOIN
        // block) and if the first_child_executed hasn't been set to true yet, set it to true
        if let Some(parent) = self.blocks.last_mut() {
            if let BlockType::Join(first_child_executed) = parent.block_type {
                if !first_child_executed {
                    parent.block_type = BlockType::Join(true);
                }
            }
        }
        block
    }

    /// Returns a reference to a block at the top of the stack.
    pub fn peek(&self) -> &BlockInfo {
        self.blocks.last().expect("block stack is empty")
    }

    /// Returns a mutable reference to a block at the top of the stack.
    pub fn peek_mut(&mut self) -> &mut BlockInfo {
        self.blocks.last_mut().expect("block stack is empty")
    }
}

/// Contains basic information about a code block.
#[derive(Debug, Clone, Copy)]
pub struct BlockInfo {
    addr: Felt,
    block_type: BlockType,
    parent_addr: Felt,
    is_loop_body: bool,
    is_first_child: bool,
}

impl BlockInfo {
    /// Returns ONE if the this block is a LOOP block and the body of the loop was executed at
    /// least once; otherwise, returns ZERO.
    pub fn is_entered_loop(&self) -> Felt {
        if self.block_type == BlockType::Loop(true) {
            ONE
        } else {
            ZERO
        }
    }

    /// Returns ONE if this block is a body of a LOOP block; otherwise returns ZERO.
    pub fn is_loop_body(&self) -> Felt {
        if self.is_loop_body {
            ONE
        } else {
            ZERO
        }
    }

    /// Returns ONE if this block is the first child of a JOIN block; otherwise returns ZERO.
    #[allow(dead_code)]
    pub fn is_first_child(&self) -> Felt {
        if self.is_first_child {
            ONE
        } else {
            ZERO
        }
    }
}

/// Specifies type of a code block with additional info for some block types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Join(bool), // internal value set to true when the first child is fully executed
    Split,
    Loop(bool), // internal value set to false if the loop is never entered
    Span,
}

impl BlockType {
    /// Returns the number of children a block has. This is an integer between 0 and 2 (both
    /// inclusive).
    pub fn num_children(&self) -> u32 {
        match self {
            Self::Join(_) => 2,
            Self::Split => 1,
            Self::Loop(is_entered) => {
                if *is_entered {
                    1
                } else {
                    0
                }
            }
            Self::Span => 0,
        }
    }
}

// SPAN CONTEXT
// ================================================================================================

/// Keeps track of the info needed to decode a currently executing SPAN block. The info includes:
/// - Operations which still need to be executed in the current group. The operations are
///   encoded as opcodes (7 bits) appended one after another into a single field element, with the
///   next operation to be executed located at the least significant position.
/// - Number of operation groups left to be executed in the entire SPAN block.
struct SpanContext {
    group_ops_left: Felt,
    num_groups_left: Felt,
}

impl Default for SpanContext {
    fn default() -> Self {
        Self {
            group_ops_left: ZERO,
            num_groups_left: ZERO,
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Removes the specified operation from the op group and returns the resulting op group.
fn remove_opcode_from_group(op_group: Felt, op: Operation) -> Felt {
    let opcode = op.op_code() as u64;
    let result = Felt::new((op_group.as_int() - opcode) >> NUM_OP_BITS);
    debug_assert!(op_group.as_int() >= result.as_int(), "op group underflow");
    result
}

/// Returns the number of op groups in the next batch based on how many total groups are left to
/// process in a span.
///
/// This is computed as the min of number of groups left and max batch size. Thus, if the number
/// of groups left is > 8, the number of groups will be 8; otherwise, it will be equal to the
/// number of groups left to process.
fn get_num_groups_in_next_batch(num_groups_left: Felt) -> usize {
    core::cmp::min(num_groups_left.as_int() as usize, OP_BATCH_SIZE)
}

// TEST HELPERS
// ================================================================================================

/// Build an operation group from the specified list of operations.
#[cfg(test)]
pub fn build_op_group(ops: &[Operation]) -> Felt {
    let mut group = 0u64;
    let mut i = 0;
    for op in ops.iter() {
        group |= (op.op_code() as u64) << (Operation::OP_BITS * i);
        i += 1;
    }
    assert!(i <= super::OP_GROUP_SIZE, "too many ops");
    Felt::new(group)
}

// DEBUG INFO
// ================================================================================================

pub struct DebugInfo {
    in_debug_mode: bool,
    operations: Vec<Operation>,
    assembly_ops: Vec<(usize, AssemblyOp)>,
}

impl DebugInfo {
    pub fn new(in_debug_mode: bool) -> Self {
        Self {
            in_debug_mode,
            operations: Vec::<Operation>::new(),
            assembly_ops: Vec::<(usize, AssemblyOp)>::new(),
        }
    }

    /// Returns whether this decoder instance is instantiated in debug mode.
    #[inline(always)]
    pub fn in_debug_mode(&self) -> bool {
        self.in_debug_mode
    }

    /// Returns an operation to be executed at the specified clock cycle. Only applicable in debug mode.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Returns list of assembly operations in debug mode.
    pub fn assembly_ops(&self) -> &[(usize, AssemblyOp)] {
        &self.assembly_ops
    }

    /// Adds an operation to the operations vector in debug mode.
    #[inline(always)]
    pub fn append_operation(&mut self, op: Operation) {
        if self.in_debug_mode {
            self.operations.push(op);
        }
    }

    /// Appends an asmop decorator at the specified clock cycle to the asmop list in debug mode.
    pub fn append_asmop(&mut self, clk: usize, asmop: AssemblyOp) {
        self.assembly_ops.push((clk, asmop));
    }
}
