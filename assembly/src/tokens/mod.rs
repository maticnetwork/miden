use super::{AssemblyError, String, ToString, Vec};
use core::fmt;

mod stream;
pub use stream::TokenStream;

// TOKEN
// ================================================================================================
/// TODO: add comments
#[derive(Clone, Debug, Default)]
pub struct Token<'a> {
    parts: Vec<&'a str>,
    pos: usize,
}

impl<'a> Token<'a> {
    // CONTROL TOKENS
    // --------------------------------------------------------------------------------------------

    pub const USE: &'static str = "use";
    pub const PROC: &'static str = "proc";
    pub const EXPORT: &'static str = "export";

    pub const BEGIN: &'static str = "begin";
    pub const IF: &'static str = "if";
    pub const ELSE: &'static str = "else";
    pub const WHILE: &'static str = "while";
    pub const REPEAT: &'static str = "repeat";
    pub const EXEC: &'static str = "exec";
    pub const END: &'static str = "end";

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new token created from the specified string and position.
    ///
    /// # Panics
    /// Panic is the `token` parameter is an empty string.
    pub fn new(token: &'a str, pos: usize) -> Self {
        assert!(!token.is_empty(), "token cannot be an empty string");
        Self {
            parts: token.split('.').collect(),
            pos,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the position of this token in the source.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the number of parts in this token.
    pub fn num_parts(&self) -> usize {
        self.parts.len()
    }

    /// Returns a reference to this token's parts.
    pub fn parts(&self) -> &[&str] {
        &self.parts
    }

    /// Returns true if this token represents a flow control token.
    pub fn is_control_token(&self) -> bool {
        matches!(
            self.parts()[0],
            Self::USE
                | Self::PROC
                | Self::EXPORT
                | Self::BEGIN
                | Self::IF
                | Self::ELSE
                | Self::WHILE
                | Self::REPEAT
                | Self::EXEC
                | Self::END
        )
    }

    // STATE MUTATOR
    // --------------------------------------------------------------------------------------------
    /// Updates the contents of this token from the specified string and position.
    ///
    /// # Panics
    /// Panic is the `token` parameter is an empty string.
    pub fn update(&mut self, token: &'a str, pos: usize) {
        assert!(!token.is_empty(), "token cannot be an empty string");
        self.parts.clear();
        token.split('.').for_each(|part| self.parts.push(part));
        self.pos = pos;
    }

    // CONTROL TOKEN PARSERS / VALIDATORS
    // --------------------------------------------------------------------------------------------

    pub fn parse_use(&self) -> Result<String, AssemblyError> {
        assert_eq!(Self::USE, self.parts[0], "not a use");
        match self.num_parts() {
            1 => Err(AssemblyError::missing_param(self)),
            2 => validate_import_path(self.parts[1], self),
            _ => Err(AssemblyError::extra_param(self)),
        }
    }

    pub fn validate_begin(&self) -> Result<(), AssemblyError> {
        assert_eq!(Self::BEGIN, self.parts[0], "not a begin");
        if self.num_parts() > 1 {
            Err(AssemblyError::extra_param(self))
        } else {
            Ok(())
        }
    }

    pub fn parse_proc(&self) -> Result<(String, u32, bool), AssemblyError> {
        assert!(
            self.parts[0] == Self::PROC || self.parts[0] == Self::EXPORT,
            "invalid procedure declaration"
        );
        let is_export = self.parts[0] == Self::EXPORT;
        match self.num_parts() {
            1 => Err(AssemblyError::missing_param(self)),
            2 => {
                let label = validate_proc_declaration_label(self.parts[1], self)?;
                Ok((label, 0, is_export))
            }
            3 => {
                let label = validate_proc_declaration_label(self.parts[1], self)?;
                let num_locals = validate_proc_locals(self.parts[2], self)?;
                Ok((label, num_locals, is_export))
            }
            _ => Err(AssemblyError::extra_param(self)),
        }
    }

    pub fn validate_if(&self) -> Result<(), AssemblyError> {
        assert_eq!(Self::IF, self.parts[0], "not an if");
        match self.num_parts() {
            1 => Err(AssemblyError::missing_param(self)),
            2 => {
                if self.parts[1] != "true" {
                    Err(AssemblyError::invalid_param(self, 1))
                } else {
                    Ok(())
                }
            }
            _ => Err(AssemblyError::extra_param(self)),
        }
    }

    pub fn validate_else(&self) -> Result<(), AssemblyError> {
        assert_eq!(Self::ELSE, self.parts[0], "not an else");
        if self.num_parts() > 1 {
            Err(AssemblyError::extra_param(self))
        } else {
            Ok(())
        }
    }

    pub fn validate_while(&self) -> Result<(), AssemblyError> {
        assert_eq!(Self::WHILE, self.parts[0], "not a while");
        match self.num_parts() {
            1 => Err(AssemblyError::missing_param(self)),
            2 => {
                if self.parts[1] != "true" {
                    Err(AssemblyError::invalid_param(self, 1))
                } else {
                    Ok(())
                }
            }
            _ => Err(AssemblyError::extra_param(self)),
        }
    }

    pub fn parse_repeat(&self) -> Result<u32, AssemblyError> {
        assert_eq!(Self::REPEAT, self.parts[0], "not a repeat");
        match self.num_parts() {
            1 => Err(AssemblyError::missing_param(self)),
            2 => self.parts[1]
                .parse::<u32>()
                .map_err(|_| AssemblyError::invalid_param(self, 1)),
            _ => Err(AssemblyError::extra_param(self)),
        }
    }

    pub fn parse_exec(&self) -> Result<String, AssemblyError> {
        assert_eq!(Self::EXEC, self.parts[0], "not an exec");
        match self.num_parts() {
            1 => Err(AssemblyError::missing_param(self)),
            2 => validate_proc_invocation_label(self.parts[1], self),
            _ => Err(AssemblyError::extra_param(self)),
        }
    }

    pub fn validate_end(&self) -> Result<(), AssemblyError> {
        assert_eq!(Self::END, self.parts[0], "not an end");
        if self.num_parts() > 1 {
            Err(AssemblyError::extra_param(self))
        } else {
            Ok(())
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.parts.join("."))
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Label of a declared procedure must comply with the following rules:
/// - It must start with an ascii letter.
/// - It can contain only ascii letters, numbers, or underscores.
fn validate_proc_declaration_label(label: &str, token: &Token) -> Result<String, AssemblyError> {
    // a label must start with a letter
    if label.is_empty() || !label.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(AssemblyError::invalid_proc_label(token, label));
    }

    // a declaration label can contain only letters, numbers, or underscores
    if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AssemblyError::invalid_proc_label(token, label));
    }

    Ok(label.to_string())
}

/// A label of an invoked procedure must comply with the following rules:
/// - It must start with an ascii letter.
/// - It can contain only ascii letters, numbers, underscores, or colons.
///
/// As compared to procedure declaration label, colons are allowed here to support invocation
/// of imported procedures.
fn validate_proc_invocation_label(label: &str, token: &Token) -> Result<String, AssemblyError> {
    // a label must start with a letter
    if label.is_empty() || !label.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(AssemblyError::invalid_proc_label(token, label));
    }

    // a label can contain only letters, numbers, underscores, or colons
    if !label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
    {
        return Err(AssemblyError::invalid_proc_label(token, label));
    }

    Ok(label.to_string())
}

fn validate_proc_locals(locals: &str, token: &Token) -> Result<u32, AssemblyError> {
    match locals.parse::<u64>() {
        Ok(num_locals) => {
            if num_locals > u32::MAX as u64 {
                return Err(AssemblyError::invalid_proc_locals(token, locals));
            }
            Ok(num_locals as u32)
        }
        Err(_) => Err(AssemblyError::invalid_proc_locals(token, locals)),
    }
}

/// A module import path must comply with the following rules:
/// - It must start with an ascii letter.
/// - It can contain only ascii letters, numbers, underscores, or colons.
fn validate_import_path(path: &str, token: &Token) -> Result<String, AssemblyError> {
    // a path must start with a letter
    if path.is_empty() || !path.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(AssemblyError::invalid_module_path(token, path));
    }

    // a path can contain only letters, numbers, underscores, or colons
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
    {
        return Err(AssemblyError::invalid_module_path(token, path));
    }

    Ok(path.to_string())
}
