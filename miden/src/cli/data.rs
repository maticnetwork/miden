use assembly::Assembler;
use prover::StarkProof;
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fs, io::Write, time::Instant};
use vm_core::{hasher::Digest, Program, ProgramInputs};
use winter_utils::{Deserializable, SliceReader};

// INPUT FILE
// ================================================================================================

/// Input file struct
#[derive(Deserialize, Debug)]
pub struct InputFile {
    pub stack_init: Vec<String>,
}

/// Helper methods to interact with the input file
impl InputFile {
    pub fn read(inputs_path: &Option<PathBuf>, program_path: &Path) -> Result<Self, String> {
        // if file not specified explicitly and corresponding file with same name as program_path
        // with '.inputs' extension does't exist, set stack_init to empty vector
        if !inputs_path.is_some() && !program_path.with_extension("inputs").exists() {
            return Ok(Self {
                stack_init: Vec::new(),
            });
        }

        // If inputs_path has been provided then use this as path. Alternatively we will
        // replace the program_path extension with `.inputs` and use this as a default.
        let path = match inputs_path {
            Some(path) => path.clone(),
            None => program_path.with_extension("inputs"),
        };

        println!("Reading input file `{}`", path.display());

        // read input file to string
        let inputs_file = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to open input file `{}` - {}", path.display(), err))?;

        // deserialize input data
        let inputs: InputFile = serde_json::from_str(&inputs_file)
            .map_err(|err| format!("Failed to deserialize input data - {}", err))?;

        Ok(inputs)
    }

    // TODO add handling of advice provider inputs
    pub fn get_program_inputs(&self) -> ProgramInputs {
        ProgramInputs::from_stack_inputs(&self.stack_init()).unwrap()
    }

    /// Parse stack_init vector of strings to a vector of u64
    pub fn stack_init(&self) -> Vec<u64> {
        self.stack_init
            .iter()
            .map(|v| v.parse::<u64>().unwrap())
            .collect::<Vec<u64>>()
    }
}

// OUTPUT FILE
// ================================================================================================

/// Output file struct
#[derive(Deserialize, Serialize, Debug)]
pub struct OutputFile {
    pub outputs: Vec<String>,
}

/// Helper methods to interact with the output file
impl OutputFile {
    /// Returns a new [OutputFile] from the specified outputs vector
    pub fn new(outputs: Vec<u64>) -> Self {
        Self {
            outputs: outputs
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
        }
    }

    /// Read the output file
    pub fn read(outputs_path: &Option<PathBuf>, program_path: &Path) -> Result<Self, String> {
        // If outputs_path has been provided then use this as path.  Alternatively we will
        // replace the program_path extension with `.outputs` and use this as a default.
        let path = match outputs_path {
            Some(path) => path.clone(),
            None => program_path.with_extension("outputs"),
        };

        println!("Reading output file `{}`", path.display());

        // read outputs file to string
        let outputs_file = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to open outputs file `{}` - {}", path.display(), err))?;

        // deserialize outputs data
        let mut outputs: OutputFile = serde_json::from_str(&outputs_file)
            .map_err(|err| format!("Failed to deserialize outputs data - {}", err))?;

        // The verify interface expects the stack outputs in reverse order so we reverse them here
        outputs.outputs.reverse();

        Ok(outputs)
    }

    /// Write the output file
    pub fn write(outputs: Vec<u64>, path: &Option<PathBuf>) -> Result<(), String> {
        if let Some(path) = path {
            // if path provided, create output file
            println!("Creating output file `{}`", path.display());

            let file = fs::File::create(&path).map_err(|err| {
                format!(
                    "Failed to create output file `{}` - {}",
                    path.display(),
                    err
                )
            })?;

            println!("Writing data to output file");

            // write outputs to output file
            serde_json::to_writer_pretty(file, &Self::new(outputs))
        } else {
            println!("Output: {:?}", outputs);
            Ok(())
        }
        .map_err(|err| format!("Failed to write output data - {}", err))
    }

    /// Converts outputs vector of String to vector of u64
    pub fn outputs(&self) -> Vec<u64> {
        self.outputs
            .iter()
            .map(|v| v.parse::<u64>().unwrap())
            .collect::<Vec<u64>>()
    }
}

// PROGRAM FILE
// ================================================================================================

pub struct ProgramFile;

/// Helper methods to interact with masm program file
impl ProgramFile {
    pub fn read(path: &PathBuf) -> Result<Program, String> {
        println!("Reading program file `{}`", path.display());

        // read program file to string
        let program_file = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to open program file `{}` - {}", path.display(), err))?;

        print!("Compiling program... ");
        let now = Instant::now();

        // compile program
        let program = Assembler::default()
            .compile(&program_file)
            .map_err(|err| format!("Failed to compile program - {}", err))?;

        println!("done ({} ms)", now.elapsed().as_millis());

        Ok(program)
    }
}

// PROOF FILE
// ================================================================================================

pub struct ProofFile;

/// Helper methods to interact with proof file
impl ProofFile {
    /// Read stark proof from file
    pub fn read(proof_path: &Option<PathBuf>, program_path: &Path) -> Result<StarkProof, String> {
        // If proof_path has been provided then use this as path.  Alternatively we will
        // replace the program_path extension with `.proof` and use this as a default.
        let path = match proof_path {
            Some(path) => path.clone(),
            None => program_path.with_extension("proof"),
        };

        println!("Reading proof file `{}`", path.display());

        // read the file to bytes
        let file = fs::read(&path)
            .map_err(|err| format!("Failed to open proof file `{}` - {}", path.display(), err))?;

        // deserialize bytes into a stark proof
        StarkProof::from_bytes(&file)
            .map_err(|err| format!("Failed to decode proof data - {}", err))
    }

    /// Write stark proof to file
    pub fn write(
        proof: StarkProof,
        proof_path: &Option<PathBuf>,
        program_path: &Path,
    ) -> Result<(), String> {
        // If proof_path has been provided then use this as path.  Alternatively we will
        // replace the program_path extension with `.proof` and use this as a default.
        let path = match proof_path {
            Some(path) => path.clone(),
            None => program_path.with_extension("proof"),
        };

        println!("Creating proof file `{}`", path.display());

        // create output fille
        let mut file = fs::File::create(&path)
            .map_err(|err| format!("Failed to create proof file `{}` - {}", path.display(), err))?;

        let proof_bytes = proof.to_bytes();

        println!(
            "Writing data to proof file - size {} KB",
            proof_bytes.len() / 1024
        );

        // write proof bytes to file
        file.write_all(&proof_bytes).unwrap();

        Ok(())
    }
}

// PROGRAM HASH
// ================================================================================================

pub struct ProgramHash;

/// Helper method to parse program hash from hex
impl ProgramHash {
    pub fn read(hash_hex_string: &String) -> Result<Digest, String> {
        // decode hex to bytes
        let program_hash_bytes = hex::decode(hash_hex_string)
            .map_err(|err| format!("Failed to convert program hash to bytes {}", err))?;

        // create slice reader from bytes
        let mut program_hash_slice = SliceReader::new(&program_hash_bytes);

        // create hash digest from slice
        let program_hash = Digest::read_from(&mut program_hash_slice)
            .map_err(|err| format!("Failed to deserialize program hash from bytes - {}", err))?;

        Ok(program_hash)
    }
}
