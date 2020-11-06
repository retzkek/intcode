extern crate intcode;
use argh::FromArgs;
use std::fs::File;
use std::io;

#[derive(FromArgs)]
/// intcode interpreter
struct Args {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Run(CommandRun),
}

#[derive(FromArgs, PartialEq, Debug)]
/// run program
#[argh(subcommand, name = "run")]
struct CommandRun {
    #[argh(positional)]
    /// source code file
    filename: String,
    #[argh(switch)]
    /// trace program execution
    trace: bool,
    #[argh(switch)]
    /// print final memory status
    print: bool,
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();
    match args.subcommand {
        Subcommand::Run(r) => {
            let f = File::open(r.filename)?;
            let reader = io::BufReader::new(f);
            let mut prog = intcode::Program::new(reader);
            prog.exe(0, r.trace);
            if r.print {
                println!["{}", prog];
            }
        }
    };
    Ok(())
}
