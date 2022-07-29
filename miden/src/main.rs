use std::io::Write;
use structopt::StructOpt;

mod cli;
mod examples;
mod tools;

/// Root CLI struct
#[derive(StructOpt, Debug)]
#[structopt(name = "Miden", about = "Miden CLI")]
pub struct Cli {
    #[structopt(subcommand)]
    action: Actions,
}

/// CLI actions
#[derive(StructOpt, Debug)]
pub enum Actions {
    Analyze(tools::Analyze),
    Compile(cli::CompileCmd),
    Example(examples::ExampleOptions),
    Prove(cli::ProveCmd),
    Run(cli::RunCmd),
    Verify(cli::VerifyCmd),
}

/// CLI entry point
impl Cli {
    pub fn execute(&self) -> Result<(), String> {
        match &self.action {
            Actions::Analyze(analyze) => analyze.execute(),
            Actions::Compile(compile) => compile.execute(),
            Actions::Example(example) => example.execute(),
            Actions::Prove(prove) => prove.execute(),
            Actions::Run(run) => run.execute(),
            Actions::Verify(verify) => verify.execute(),
        }
    }
}

/// Executable entry point
pub fn main() {
    // configure logging
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Debug)
        .init();

    // read command-line args
    let cli = Cli::from_args();

    // execute cli action
    if let Err(error) = cli.execute() {
        println!("{}", error);
    }
}
