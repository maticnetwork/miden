use std::collections::BTreeMap;
use std::io::Result;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

mod stdlib_docs;

// CONSTANTS
// ================================================================================================

const ASM_DIR_PATH: &str = "./asm";
const DOC_DIR_PATH: &str = "./docs";
const ASM_FILE_PATH: &str = "./src/asm.rs";

// TYPE ALIASES
// ================================================================================================

type ModuleMap = BTreeMap<String, String>;

// PRE-PROCESSING
// ================================================================================================

/// Reads the contents of the `./asm` directory, and writes these contents into a single `.rs`
/// file under `./src/asm.rs`.
///
/// The `asm.rs` file exports a single static array of string tuples. Each tuple consist of module
/// namespace label and the module source code for this label.
#[cfg(not(feature = "docs-rs"))]
fn main() {
    // re-build the `./src/asm.rs` file only if something in the `./asm` directory has changed
    println!("cargo:rerun-if-changed=asm");

    let mut modules = BTreeMap::new();

    // read the modules from the asm directory
    let path = Path::new(ASM_DIR_PATH);
    read_modules(path, "std".to_string(), &mut modules)
        .expect("failed to read modules from the asm directory");

    // write the modules into the asm file
    write_asm_rs(&modules).expect("failed to write modules into the module file");

    // updates the documentation of these modules
    stdlib_docs::build_stdlib_docs(&modules, DOC_DIR_PATH);
}

// HELPER FUNCTIONS
// ================================================================================================

/// Recursively reads Miden assembly modules from the specified path, and inserts the modules
/// to the provided module map.
fn read_modules(fs_path: &Path, ns_path: String, modules: &mut ModuleMap) -> Result<()> {
    // iterate over all entries in the directory
    for dir in fs_path.read_dir()? {
        let path = dir?.path();

        if path.is_dir() {
            // if the current path is a directory, continue reading it recursively
            let dir_name = path
                .file_name()
                .expect("failed to get directory name from path")
                .to_str()
                .expect("failed to convert directory name to string");
            let ns_path = format!("{}::{}", ns_path, dir_name);
            read_modules(path.as_path(), ns_path, modules)?;
        } else if path.is_file() {
            // if the current path is a file, make sure it is a `.masm` file and read its contents
            let extension = path
                .extension()
                .expect("failed to get file extension from path")
                .to_str()
                .expect("failed to convert file extension to string");
            assert_eq!("masm", extension, "invalid file extension at: {:?}", path);
            let source = fs::read_to_string(path.as_path())?;

            // get the name of the file without extension
            let file_name = path
                .with_extension("") // strip te extension
                .as_path()
                .file_name()
                .expect("failed to get file name from path")
                .to_str()
                .expect("failed to convert file name to string")
                .to_string();
            // insert the module source into the module map
            modules.insert(format!("{}::{}", ns_path, file_name), source);
        } else {
            panic!("entry not a file or directory");
        }
    }

    Ok(())
}

/// Writes Miden assembly modules into a single `asm.rs` file.
#[rustfmt::skip]
fn write_asm_rs(modules: &ModuleMap) -> Result<()> {
    // create the module file
    let mut asm_file = File::create(ASM_FILE_PATH)?;

    // write module header which also opens the array
    writeln!(asm_file, "//! This module is automatically generated during build time and should not be modified manually.\n")?;
    writeln!(asm_file, "/// An array of modules defined in Miden standard library.")?;
    writeln!(asm_file, "///")?;
    writeln!(asm_file, "/// Entries in the array are tuples containing module namespace and module source code.")?;
    writeln!(asm_file, "#[rustfmt::skip]")?;
    writeln!(asm_file, "pub const MODULES: [(&str, &str); {}] = [", modules.len())?;

    // write each module into the module file
    for (ns, source) in modules {
        let separator_suffix = (0..(89 - ns.len())).map(|_| "-").collect::<String>();
        writeln!(asm_file, "// ----- {} {}", ns, separator_suffix)?;
        writeln!(asm_file, "(\"{}\", \"{}\"),", ns, source)?;
    }

    // close the array
    writeln!(asm_file, "];")
}
