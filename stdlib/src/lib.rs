#![cfg_attr(not(feature = "std"), no_std)]

use vm_core::{
    errors::LibraryError,
    utils::{collections::BTreeMap, string::ToString},
    Library,
};

mod asm;
use asm::MODULES;

// CONSTANTS
// ================================================================================================

const VERSION: &str = env!("CARGO_PKG_VERSION");

// TYPE ALIASES
// ================================================================================================

type ModuleMap = BTreeMap<&'static str, &'static str>;

// STANDARD LIBRARY
// ================================================================================================

/// TODO: add docs
pub struct StdLibrary {
    modules: ModuleMap,
}

impl Library for StdLibrary {
    /// Returns root namespace of the standard library, which is always "std".
    fn root_ns(&self) -> &str {
        "std"
    }

    /// Returns the current version of the standard library.
    fn version(&self) -> &str {
        VERSION
    }

    /// Returns the source code of the module located at the specified path.
    ///
    /// # Errors
    /// Returns an error if the modules for the specified path does not exist in the standard
    /// library.
    fn get_module_source(&self, module_path: &str) -> Result<&str, LibraryError> {
        let source = self
            .modules
            .get(module_path)
            .ok_or_else(|| LibraryError::ModuleNotFound(module_path.to_string()))?;
        Ok(source)
    }
}

impl Default for StdLibrary {
    /// Returns a new [StdLibrary] instance instantiated with default parameters.
    fn default() -> Self {
        let mut modules = BTreeMap::new();
        for (ns, source) in MODULES {
            modules.insert(ns, source);
        }
        Self { modules }
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::Library;

    #[test]
    fn lib_version() {
        let stdlib = super::StdLibrary::default();
        assert_eq!("0.1.0", stdlib.version())
    }
}
