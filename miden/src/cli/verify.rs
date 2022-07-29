use super::data::{InputFile, OutputFile, ProgramHash, ProofFile};
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Verify", about = "Verify a miden program")]
pub struct VerifyCmd {
    /// Path to input file
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input_file: Option<PathBuf>,
    /// Path to output file
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output_file: Option<PathBuf>,
    /// Path to proof file
    #[structopt(short = "p", long = "proof", parse(from_os_str))]
    proof_file: PathBuf,
    /// Program hash (hex)
    #[structopt(short = "h", long = "program-hash")]
    program_hash: String,
}

impl VerifyCmd {
    pub fn execute(&self) -> Result<(), String> {
        println!("============================================================");
        println!("Verify program");
        println!("============================================================");

        // read program hash from input
        let program_hash = ProgramHash::read(&self.program_hash)?;

        // load input data from file
        let input_data = InputFile::read(&self.input_file, &self.proof_file)?;

        // load outputs data from file
        let outputs_data = OutputFile::read(&self.output_file, &self.proof_file)?;

        // load proof from file
        let proof = ProofFile::read(&Some(self.proof_file.clone()), &self.proof_file)?;

        println!("verifying program...");
        let now = Instant::now();

        // verify proof
        verifier::verify(
            program_hash,
            &input_data.stack_init(),
            &outputs_data.outputs(),
            proof,
        )
        .map_err(|err| format!("Program failed verification! - {}", err))?;

        println!("Verification complete in {} ms", now.elapsed().as_millis());

        Ok(())
    }
}
