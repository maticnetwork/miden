#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

use vm_core::{
    code_blocks::CodeBlock,
    utils::{
        collections::{BTreeMap, Vec},
        string::{String, ToString},
    },
    Library, Program,
};
use vm_stdlib::StdLibrary;

mod context;
use context::AssemblyContext;

mod procedures;
use procedures::Procedure;

mod parsers;
use parsers::{combine_blocks, parse_code_blocks};

mod tokens;
use tokens::{Token, TokenStream};

mod errors;
pub use errors::AssemblyError;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const MODULE_PATH_DELIM: &str = "::";

// TYPE ALIASES
// ================================================================================================

type ProcMap = BTreeMap<String, Procedure>;
type ModuleMap = BTreeMap<String, ProcMap>;

// ASSEMBLER
// ================================================================================================

/// TODO: add comments
pub struct Assembler {
    stdlib: StdLibrary,
    parsed_modules: ModuleMap,
    in_debug_mode: bool,
}

impl Assembler {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new instance of [Assembler] instantiated with empty module map.
    /// Debug related decorators are added to span blocks when debug mode is on.
    pub fn new(in_debug_mode: bool) -> Self {
        Self {
            stdlib: StdLibrary::default(),
            parsed_modules: BTreeMap::new(),
            in_debug_mode,
        }
    }

    // PROGRAM COMPILER
    // --------------------------------------------------------------------------------------------

    /// Compiles the provided source code into a [Program]. The resulting program can be executed
    /// on Miden VM.
    pub fn compile(&self, source: &str) -> Result<Program, AssemblyError> {
        let mut tokens = TokenStream::new(source)?;
        let mut context = AssemblyContext::new();

        // parse imported modules (if any), and add exported procedures from these modules to the
        // current context; since we are in the root context here, we initialize dependency chain
        // with an empty vector.
        self.parse_imports(&mut tokens, &mut context, &mut Vec::new())?;

        // parse locally defined procedures (if any), and add these procedures to the current
        // context
        while let Some(token) = tokens.read() {
            let proc = match token.parts()[0] {
                Token::PROC | Token::EXPORT => {
                    Procedure::parse(&mut tokens, &context, false, self.in_debug_mode)?
                }
                _ => break,
            };
            context.add_local_proc(proc);
        }

        // make sure program body is present
        let next_token = tokens
            .read()
            .ok_or_else(|| AssemblyError::unexpected_eof(tokens.pos()))?;
        if next_token.parts()[0] != Token::BEGIN {
            return Err(AssemblyError::unexpected_token(next_token, Token::BEGIN));
        }

        // parse program body and return the resulting program
        let program_root = parse_program(&mut tokens, &context, self.in_debug_mode)?;
        Ok(Program::new(program_root))
    }

    // IMPORT PARSERS
    // --------------------------------------------------------------------------------------------

    /// Parses `use` instructions from the token stream.
    ///
    /// For each `use` instructions, retrieves exported procedures from the specified module and
    /// inserts them into the provided context.
    ///
    /// If a module specified by `use` instruction hasn't been parsed yet, parses it, and adds
    /// the parsed module to `self.parsed_modules`.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The `use` instruction is malformed.
    /// - A module specified by the `use` instruction could not be found.
    /// - Parsing the specified module results in an error.
    fn parse_imports<'a>(
        &'a self,
        tokens: &mut TokenStream,
        context: &mut AssemblyContext<'a>,
        dep_chain: &mut Vec<String>,
    ) -> Result<(), AssemblyError> {
        // read tokens from the token stream until all `use` tokens are consumed
        while let Some(token) = tokens.read() {
            match token.parts()[0] {
                Token::USE => {
                    // parse the `use` instruction to extract module path from it
                    let module_path = &token.parse_use()?;

                    // check if a module with the same path is currently being parsed somewhere up
                    // the chain; if it is, then we have a circular dependency.
                    if dep_chain.iter().any(|v| v == module_path) {
                        dep_chain.push(module_path.clone());
                        return Err(AssemblyError::circular_module_dependency(token, dep_chain));
                    }

                    // add the current module to the dependency chain
                    dep_chain.push(module_path.clone());

                    // if the module hasn't been parsed yet, retrieve its source from the library
                    // and attempt to parse it; if the parsing is successful, this will also add
                    // the parsed module to `self.parsed_modules`
                    if !self.parsed_modules.contains_key(module_path) {
                        let module_source =
                            self.stdlib.get_module_source(module_path).map_err(|_| {
                                AssemblyError::missing_import_source(token, module_path)
                            })?;
                        self.parse_module(module_source, module_path, dep_chain)?;
                    }

                    // get procedures from the module at the specified path; we are guaranteed to
                    // not fail here because the above code block ensures that either there is a
                    // parsed module for the specified path, or the function returns with an error
                    let module_procs = self
                        .parsed_modules
                        .get(module_path)
                        .expect("no module procs");

                    // add all procedures to the current context; procedure labels are set to be
                    // `last_part_of_module_path::procedure_name`. For example, `u256::add`.
                    for proc in module_procs.values() {
                        let path_parts = module_path.split(MODULE_PATH_DELIM).collect::<Vec<_>>();
                        let num_parts = path_parts.len();
                        context.add_imported_proc(path_parts[num_parts - 1], proc);
                    }

                    // consume the `use` token and pop the current module of the dependency chain
                    tokens.advance();
                    dep_chain.pop();
                }
                _ => break,
            }
        }

        Ok(())
    }

    /// Parses a set of exported procedures from the specified source code and adds these
    /// procedures to `self.parsed_modules` using the specified path as the key.
    #[allow(clippy::cast_ref_to_mut)]
    fn parse_module(
        &self,
        source: &str,
        path: &str,
        dep_chain: &mut Vec<String>,
    ) -> Result<(), AssemblyError> {
        let mut tokens = TokenStream::new(source)?;
        let mut context = AssemblyContext::new();

        // parse imported modules (if any), and add exported procedures from these modules to
        // the current context
        self.parse_imports(&mut tokens, &mut context, dep_chain)?;

        // parse procedures defined in the module, and add these procedures to the current
        // context
        while let Some(token) = tokens.read() {
            let proc = match token.parts()[0] {
                Token::PROC | Token::EXPORT => {
                    Procedure::parse(&mut tokens, &context, true, self.in_debug_mode)?
                }
                _ => break,
            };
            context.add_local_proc(proc);
        }

        // make sure there are no dangling instructions after all procedures have been read
        if !tokens.eof() {
            let token = tokens.read().expect("no token before eof");
            return Err(AssemblyError::dangling_ops_after_module(token, path));
        }

        // extract the exported local procedures from the context
        let mut module_procs = context.into_local_procs();
        module_procs.retain(|_, p| p.is_export());

        // insert exported procedures into `self.parsed_procedures`
        // TODO: figure out how to do this using interior mutability
        unsafe {
            let path = path.to_string();
            let mutable_self = &mut *(self as *const _ as *mut Assembler);
            mutable_self.parsed_modules.insert(path, module_procs);
        }

        Ok(())
    }
}

impl Default for Assembler {
    /// Returns a new instance of [Assembler] instantiated with empty module map in non-debug mode.
    fn default() -> Self {
        Self::new(false)
    }
}

// PARSERS
// ================================================================================================

/// TODO: add comments
fn parse_program(
    tokens: &mut TokenStream,
    context: &AssemblyContext,
    in_debug_mode: bool,
) -> Result<CodeBlock, AssemblyError> {
    let program_start = tokens.pos();
    // consume the 'begin' token
    let header = tokens.read().expect("missing program header");
    header.validate_begin()?;
    tokens.advance();

    // parse the program body
    let root = parse_code_blocks(tokens, context, 0, in_debug_mode)?;

    // consume the 'end' token
    match tokens.read() {
        None => Err(AssemblyError::unmatched_begin(
            tokens.read_at(program_start).expect("no begin token"),
        )),
        Some(token) => match token.parts()[0] {
            Token::END => token.validate_end(),
            Token::ELSE => Err(AssemblyError::dangling_else(token)),
            _ => Err(AssemblyError::unmatched_begin(
                tokens.read_at(program_start).expect("no begin token"),
            )),
        },
    }?;
    tokens.advance();

    // make sure there are no instructions after the end
    if let Some(token) = tokens.read() {
        return Err(AssemblyError::dangling_ops_after_program(token));
    }

    Ok(root)
}
