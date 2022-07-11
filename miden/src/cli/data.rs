use assembly::Assembler;
use prover::StarkProof;
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fs, io::Write, time::Instant};
use vm_core::{hasher::Digest, program::Script, ProgramInputs};
use winter_utils::{Deserializable, SliceReader};

// INPUT FILE
// ================================================================================================

/// Input file struct
#[derive(Deserialize, Debug)]
pub struct InputFile {
    pub stack_inputs: Vec<u64>,
}

/// Helper methods to interact with the input file
impl InputFile {
    pub fn read(inputs_path: &Option<PathBuf>, script_path: &Path) -> Result<Self, String> {
        // If inputs_path has been provided then use this as path.  Alternatively we will
        // replace the script_path extension with `.inputs` and use this as a default.
        let path = match inputs_path {
            Some(path) => path.clone(),
            None => script_path.with_extension("inputs"),
        };

        println!("Reading input file `{}`", path.display());

        // read input file to string
        let inputs_file = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to open input file `{}` - {}", path.display(), err))?;

        // deserilaise input data
        let inputs: InputFile = serde_json::from_str(&inputs_file)
            .map_err(|err| format!("Failed to deserialse input data - {}", err))?;

        Ok(inputs)
    }

    // TODO add handling of advice provider inputs
    pub fn get_program_inputs(&self) -> ProgramInputs {
        ProgramInputs::from_stack_inputs(&self.stack_inputs).unwrap()
    }
}

// OUTPUT FILE
// ================================================================================================

/// Output file struct
#[derive(Deserialize, Serialize, Debug)]
pub struct OutputFile {
    pub outputs: Vec<u64>,
}

/// Helper methods to interact with the output file
impl OutputFile {
    /// read the input file
    pub fn read(outputs_path: &Option<PathBuf>, script_path: &Path) -> Result<Self, String> {
        // If outputs_path has been provided then use this as path.  Alternatively we will
        // replace the script_path extension with `.outputs` and use this as a default.
        let path = match outputs_path {
            Some(path) => path.clone(),
            None => script_path.with_extension("outputs"),
        };

        println!("Reading output file `{}`", path.display());

        // read outputs file to string
        let outputs_file = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to open outputs file `{}` - {}", path.display(), err))?;

        // deserilaise outputs data
        let mut outputs: OutputFile = serde_json::from_str(&outputs_file)
            .map_err(|err| format!("Failed to deserialse outputs data - {}", err))?;

        // The verify interface exepects the stack outputs in reverse order so we reverse them here
        outputs.outputs.reverse();

        Ok(outputs)
    }

    /// write the output file
    pub fn write(outputs: Vec<u64>, path: &Option<PathBuf>) -> Result<(), String> {
        match path {
            Some(path) => {
                // if path provided create ouptut file

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
                serde_json::to_writer_pretty(file, &Self { outputs })
            }

            // no path provided - write outputs to stdout
            None => serde_json::to_writer_pretty(std::io::stdout(), &Self { outputs }),
        }
        .map_err(|err| format!("Failed to write output data - {}", err))
    }
}

// SCRIPT FILE
// ================================================================================================

pub struct ScriptFile;

/// Helper methods to interact with masm script file
impl ScriptFile {
    pub fn read(path: &PathBuf) -> Result<Script, String> {
        println!("Reading script file `{}`", path.display());

        // read script file to string
        let script_file = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to open script file `{}` - {}", path.display(), err))?;

        print!("Compiling script... ");
        let now = Instant::now();

        // compile script
        let script = Assembler::default()
            .compile_script(&script_file)
            .map_err(|err| format!("Failed to compile script - {}", err))?;

        println!("done ({} ms)", now.elapsed().as_millis());

        Ok(script)
    }
}

// PROOF FILE
// ================================================================================================

pub struct ProofFile;

/// Helper methods to interact with proof file
impl ProofFile {
    /// Read stark proof from file
    pub fn read(proof_path: &Option<PathBuf>, script_path: &Path) -> Result<StarkProof, String> {
        // If proof_path has been provided then use this as path.  Alternatively we will
        // replace the script_path extension with `.proof` and use this as a default.
        let path = match proof_path {
            Some(path) => path.clone(),
            None => script_path.with_extension("proof"),
        };

        println!("Reading proof file `{}`", path.display());

        // read the file to bytes
        let file = fs::read(&path)
            .map_err(|err| format!("Failed to open proof file `{}` - {}", path.display(), err))?;

        // deserialise bytes into a stark proof
        StarkProof::from_bytes(&file)
            .map_err(|err| format!("Failed to decode proof data - {}", err))
    }

    /// Write stark proof to file
    pub fn write(
        proof: StarkProof,
        proof_path: &Option<PathBuf>,
        script_path: &Path,
    ) -> Result<(), String> {
        // If proof_path has been provided then use this as path.  Alternatively we will
        // replace the script_path extension with `.proof` and use this as a default.
        let path = match proof_path {
            Some(path) => path.clone(),
            None => script_path.with_extension("proof"),
        };

        println!("Creating proof file `{}`", path.display());

        // create ouptut fille
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
            .map_err(|err| format!("Failed to deserialise program hash from bytes - {}", err))?;

        Ok(program_hash)
    }
}