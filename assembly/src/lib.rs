use vm_core::program::{blocks::CodeBlock, Script};
use winter_utils::collections::BTreeMap;

mod parsers;
use parsers::{parse_code_blocks, parse_proc_blocks};

mod tokens;
use tokens::{Token, TokenStream};

mod errors;
pub use errors::AssemblyError;

#[cfg(test)]
mod tests;

// ASSEMBLER
// ================================================================================================

/// TODO: add comments
pub struct Assembler {}

impl Assembler {
    pub fn new() -> Assembler {
        Self {}
    }

    /// TODO: add comments
    pub fn compile_script(&self, source: &str) -> Result<Script, AssemblyError> {
        let mut tokens = TokenStream::new(source)?;
        let mut proc_map = BTreeMap::new();

        // parse procedures and add them to the procedure map; procedures are parsed in the order
        // in which they appear in the source, and thus, procedures which come later may invoke
        // preceding procedures
        while let Some(token) = tokens.read() {
            match token.parts()[0] {
                Token::PROC => parse_proc(&mut tokens, &mut proc_map)?,
                _ => break,
            }
        }

        // make sure script body is present
        let next_token = tokens
            .read()
            .ok_or_else(|| AssemblyError::unexpected_eof(tokens.pos()))?;
        if next_token.parts()[0] != Token::BEGIN {
            return Err(AssemblyError::unexpected_token(next_token, Token::BEGIN));
        }

        // parse script body and return the resulting script
        let script_root = parse_script(&mut tokens, &proc_map)?;
        Ok(Script::new(script_root))
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

// PARSERS
// ================================================================================================

/// TODO: add comments
fn parse_script(
    tokens: &mut TokenStream,
    proc_map: &BTreeMap<String, CodeBlock>,
) -> Result<CodeBlock, AssemblyError> {
    let script_start = tokens.pos();
    // consume the 'begin' token
    let header = tokens.read().expect("missing script header");
    header.validate_begin()?;
    tokens.advance();

    // parse the script body
    let root = parse_code_blocks(tokens, proc_map, 0)?;

    // consume the 'end' token
    match tokens.read() {
        None => Err(AssemblyError::unmatched_begin(
            tokens.read_at(script_start).expect("no begin token"),
        )),
        Some(token) => match token.parts()[0] {
            Token::END => token.validate_end(),
            Token::ELSE => Err(AssemblyError::dangling_else(token)),
            _ => Err(AssemblyError::unmatched_begin(
                tokens.read_at(script_start).expect("no begin token"),
            )),
        },
    }?;
    tokens.advance();

    // make sure there are no instructions after the end
    if let Some(token) = tokens.read() {
        return Err(AssemblyError::dangling_ops_after_script(token));
    }

    Ok(root)
}

/// TODO: add comments
fn parse_proc(
    tokens: &mut TokenStream,
    proc_map: &mut BTreeMap<String, CodeBlock>,
) -> Result<(), AssemblyError> {
    let proc_start = tokens.pos();

    // read procedure name and consume the procedure header token
    let header = tokens.read().expect("missing procedure header");
    let (label, num_locals) = header.parse_proc()?;
    if proc_map.contains_key(&label) {
        return Err(AssemblyError::duplicate_proc_label(header, &label));
    }
    tokens.advance();

    // parse procedure body, and handle memory allocation/deallocation of locals if any are declared
    let root = parse_proc_blocks(tokens, proc_map, &label, num_locals)?;

    // consume the 'end' token
    match tokens.read() {
        None => Err(AssemblyError::unmatched_proc(
            tokens.read_at(proc_start).expect("no proc token"),
        )),
        Some(token) => match token.parts()[0] {
            Token::END => token.validate_end(),
            _ => Err(AssemblyError::unmatched_proc(
                tokens.read_at(proc_start).expect("no proc token"),
            )),
        },
    }?;
    tokens.advance();

    // add the procedure to the procedure map and return
    proc_map.insert(label, root);
    Ok(())
}
