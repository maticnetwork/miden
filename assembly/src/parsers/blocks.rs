use super::{
    parse_op_token, AssemblyContext, AssemblyError, CodeBlock, Operation, String, Token,
    TokenStream, Vec,
};
use vm_core::{
    utils::{group_vector_elements, string::ToString},
    Decorator, DecoratorList, ProcMarker,
};

// BLOCK PARSER
// ================================================================================================

/// TODO: Add comments
pub fn parse_code_blocks(
    tokens: &mut TokenStream,
    context: &AssemblyContext,
    num_proc_locals: u32,
    in_debug_mode: bool,
) -> Result<CodeBlock, AssemblyError> {
    // make sure there is something to be read
    let start_pos = tokens.pos();
    if tokens.eof() {
        return Err(AssemblyError::unexpected_eof(start_pos));
    }

    // parse the sequence of blocks and add each block to the list
    let mut blocks = Vec::new();
    while let Some(parser) = BlockParser::next(tokens)? {
        let block = parser.parse(tokens, context, num_proc_locals, in_debug_mode)?;
        blocks.push(block);
    }

    // make sure at least one block has been read
    if blocks.is_empty() {
        let start_op = tokens.read_at(start_pos).expect("no start token");
        Err(AssemblyError::empty_block(start_op))
    } else {
        // build a binary tree out of the parsed list of blocks
        Ok(combine_blocks(blocks))
    }
}

// CODE BLOCK PARSER
// ================================================================================================

// TODO: add comments
#[derive(Debug)]
enum BlockParser {
    Span,
    IfElse,
    While,
    Repeat(u32),
    Exec(String),
}

impl BlockParser {
    // TODO: add comments
    pub fn parse(
        &self,
        tokens: &mut TokenStream,
        context: &AssemblyContext,
        num_proc_locals: u32,
        in_debug_mode: bool,
    ) -> Result<CodeBlock, AssemblyError> {
        match self {
            Self::Span => {
                // --------------------------------------------------------------------------------
                let mut span_ops = Vec::new();
                let mut decorators = DecoratorList::new();
                while let Some(op) = tokens.read() {
                    if op.is_control_token() {
                        break;
                    }
                    parse_op_token(
                        op,
                        &mut span_ops,
                        num_proc_locals,
                        &mut decorators,
                        in_debug_mode,
                    )?;
                    tokens.advance();
                }
                Ok(CodeBlock::new_span_with_decorators(span_ops, decorators))
            }
            Self::IfElse => {
                // --------------------------------------------------------------------------------
                // record start of the if-else block and consume the 'if' token
                let if_start = tokens.pos();
                tokens.advance();

                // read the `if` clause
                let t_branch = parse_code_blocks(tokens, context, num_proc_locals, in_debug_mode)?;

                // build the `else` clause; if the else clause is specified, then read it;
                // otherwise, set to a Span with a single noop
                let f_branch = match tokens.read() {
                    Some(token) => match token.parts()[0] {
                        Token::ELSE => {
                            // record start of the `else` block and consume the `else` token
                            token.validate_else()?;
                            let else_start = tokens.pos();
                            tokens.advance();

                            // parse the `false` branch
                            let f_branch =
                                parse_code_blocks(tokens, context, num_proc_locals, in_debug_mode)?;

                            // consume the `end` token
                            match tokens.read() {
                                None => Err(AssemblyError::unmatched_else(
                                    tokens.read_at(else_start).expect("no else token"),
                                )),
                                Some(token) => match token.parts()[0] {
                                    Token::END => token.validate_end(),
                                    Token::ELSE => Err(AssemblyError::dangling_else(token)),
                                    _ => Err(AssemblyError::unmatched_else(
                                        tokens.read_at(else_start).expect("no else token"),
                                    )),
                                },
                            }?;
                            tokens.advance();

                            // return the `false` branch
                            f_branch
                        }
                        Token::END => {
                            // consume the `end` token
                            token.validate_end()?;
                            tokens.advance();

                            // when no `else` clause was specified, a Span with a single noop
                            CodeBlock::new_span(vec![Operation::Noop])
                        }
                        _ => {
                            return Err(AssemblyError::unmatched_if(
                                tokens.read_at(if_start).expect("no if token"),
                            ))
                        }
                    },
                    None => {
                        return Err(AssemblyError::unmatched_if(
                            tokens.read_at(if_start).expect("no if token"),
                        ))
                    }
                };

                Ok(CodeBlock::new_split(t_branch, f_branch))
            }
            Self::While => {
                // --------------------------------------------------------------------------------
                // record start of the while block and consume the 'while' token
                let while_start = tokens.pos();
                tokens.advance();

                // read the loop body
                let loop_body = parse_code_blocks(tokens, context, num_proc_locals, in_debug_mode)?;

                // consume the `end` token
                match tokens.read() {
                    None => Err(AssemblyError::unmatched_while(
                        tokens.read_at(while_start).expect("no if token"),
                    )),
                    Some(token) => match token.parts()[0] {
                        Token::END => token.validate_end(),
                        Token::ELSE => Err(AssemblyError::dangling_else(token)),
                        _ => Err(AssemblyError::unmatched_while(
                            tokens.read_at(while_start).expect("no if token"),
                        )),
                    },
                }?;
                tokens.advance();

                Ok(CodeBlock::new_loop(loop_body))
            }
            Self::Repeat(iter_count) => {
                // --------------------------------------------------------------------------------
                // record start of the repeat block and consume the 'repeat' token
                let repeat_start = tokens.pos();
                tokens.advance();

                // read the loop body
                let loop_body = parse_code_blocks(tokens, context, num_proc_locals, in_debug_mode)?;

                // consume the `end` token
                match tokens.read() {
                    None => Err(AssemblyError::unmatched_repeat(
                        tokens.read_at(repeat_start).expect("no repeat token"),
                    )),
                    Some(token) => match token.parts()[0] {
                        Token::END => token.validate_end(),
                        Token::ELSE => Err(AssemblyError::dangling_else(token)),
                        _ => Err(AssemblyError::unmatched_repeat(
                            tokens.read_at(repeat_start).expect("no repeat token"),
                        )),
                    },
                }?;
                tokens.advance();

                // if the body of the loop consists of a single span, unroll the loop as a single
                // span; otherwise unroll the loop as a sequence of join blocks
                if let CodeBlock::Span(span) = loop_body {
                    Ok(CodeBlock::Span(span.replicate(*iter_count as usize)))
                } else {
                    // TODO: transform the loop to a while loop instead?
                    let blocks = (0..*iter_count)
                        .map(|_| loop_body.clone())
                        .collect::<Vec<_>>();
                    Ok(combine_blocks(blocks))
                }
            }
            Self::Exec(label) => {
                // --------------------------------------------------------------------------------
                // retrieve the procedure block from the proc map and consume the 'exec' token
                let (proc_root, num_locals) = context.get_proc_code(label).ok_or_else(|| {
                    AssemblyError::undefined_proc(tokens.read().expect("no exec token"), label)
                })?;
                let mut proc_root_mut = proc_root.clone();
                tokens.advance();
                if in_debug_mode {
                    add_proc_marker(&mut proc_root_mut, label.to_string(), num_locals);
                }
                Ok(proc_root_mut)
            }
        }
    }

    // TODO: add comments
    fn next(tokens: &mut TokenStream) -> Result<Option<Self>, AssemblyError> {
        let parser = match tokens.read() {
            None => None,
            Some(token) => match token.parts()[0] {
                Token::IF => {
                    token.validate_if()?;
                    Some(Self::IfElse)
                }
                Token::ELSE => {
                    token.validate_else()?;
                    None
                }
                Token::WHILE => {
                    token.validate_while()?;
                    Some(Self::While)
                }
                Token::REPEAT => {
                    let iter_count = token.parse_repeat()?;
                    Some(Self::Repeat(iter_count))
                }
                Token::EXEC => {
                    let label = token.parse_exec()?;
                    Some(Self::Exec(label))
                }
                Token::END => {
                    token.validate_end()?;
                    None
                }
                Token::USE | Token::EXPORT | Token::PROC | Token::BEGIN => None,
                _ => Some(Self::Span),
            },
        };

        Ok(parser)
    }
}

// UTILITY FUNCTIONS
// ================================================================================================

pub fn combine_blocks(mut blocks: Vec<CodeBlock>) -> CodeBlock {
    // merge consecutive Span blocks.
    let mut merged_blocks: Vec<CodeBlock> = Vec::with_capacity(blocks.len());
    // Keep track of all the consecutive Span blocks and are merged together when
    // there is a discontinuity.
    let mut contiguous_spans: Vec<CodeBlock> = Vec::new();

    blocks.drain(0..).for_each(|block| {
        if block.is_span() {
            contiguous_spans.push(block);
        } else {
            if !contiguous_spans.is_empty() {
                merged_blocks.push(combine_spans(&mut contiguous_spans));
            }
            merged_blocks.push(block);
        }
    });
    if !contiguous_spans.is_empty() {
        merged_blocks.push(combine_spans(&mut contiguous_spans));
    }

    // build a binary tree of blocks joining them using Join blocks
    let mut blocks = merged_blocks;
    while blocks.len() > 1 {
        let last_block = if blocks.len() % 2 == 0 {
            None
        } else {
            blocks.pop()
        };

        let mut grouped_blocks = Vec::new();
        core::mem::swap(&mut blocks, &mut grouped_blocks);
        let mut grouped_blocks = group_vector_elements::<CodeBlock, 2>(grouped_blocks);
        grouped_blocks.drain(0..).for_each(|pair| {
            blocks.push(CodeBlock::new_join(pair));
        });

        if let Some(block) = last_block {
            blocks.push(block);
        }
    }

    blocks.remove(0)
}

/// Returns a CodeBlock [Span] from sequence of Span blocks provided as input.
pub fn combine_spans(spans: &mut Vec<CodeBlock>) -> CodeBlock {
    if spans.len() == 1 {
        return spans.remove(0);
    }

    let mut ops = Vec::<Operation>::new();
    let mut decorators = DecoratorList::new();
    spans.drain(0..).for_each(|block| {
        if let CodeBlock::Span(span) = block {
            for decorator in span.decorators() {
                decorators.push((decorator.0 + ops.len(), decorator.1.clone()));
            }
            for batch in span.op_batches() {
                ops.extend_from_slice(batch.ops());
            }
        } else {
            panic!(
                "Codeblock was expected to be a Span Block, got {:?}.",
                block
            );
        }
    });
    CodeBlock::new_span_with_decorators(ops, decorators)
}

/// Helper function to add ProcMarker decorator to the proc root block when "exec" token is encountered.
fn add_proc_marker(proc_root: &mut CodeBlock, label: String, num_locals: u32) {
    match proc_root {
        CodeBlock::Span(block) => {
            block.set_proc_start(
                Decorator::ProcMarker(ProcMarker::ProcStarted(label, num_locals)),
                0,
            );
            block.set_proc_end(
                Decorator::ProcMarker(ProcMarker::ProcEnded),
                block.get_ops().len() - 1,
            );
        }
        CodeBlock::Join(block) => block.append_proc_marker(Decorator::ProcMarker(
            ProcMarker::ProcStarted(label, num_locals),
        )),
        CodeBlock::Split(block) => block.append_proc_marker(Decorator::ProcMarker(
            ProcMarker::ProcStarted(label, num_locals),
        )),
        CodeBlock::Loop(block) => block.append_proc_marker(Decorator::ProcMarker(
            ProcMarker::ProcStarted(label, num_locals),
        )),
        _ => (),
    }
}
