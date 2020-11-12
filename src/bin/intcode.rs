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
    Debug(CommandDebug),
}

#[derive(FromArgs, PartialEq, Debug)]
/// run program
#[argh(subcommand, name = "run")]
struct CommandRun {
    #[argh(positional)]
    /// source code file
    filename: String,
    #[argh(switch, short = 't')]
    /// trace program execution
    trace: bool,
    #[argh(switch, short = 'p')]
    /// print final memory status
    print: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// interactively debug program
#[argh(subcommand, name = "debug")]
struct CommandDebug {
    #[argh(positional)]
    /// source code file
    filename: String,
}

fn main() -> io::Result<()> {
    let args: Args = argh::from_env();
    match args.subcommand {
        Subcommand::Run(r) => {
            let f = File::open(r.filename)?;
            let reader = io::BufReader::new(f);
            let mut prog = intcode::Program::new(reader);
            prog.exe(
                0,
                r.trace,
                intcode::Input::Reader(&mut io::stdin().lock()),
                intcode::Output::Writer(&mut io::stdout().lock()),
            )?;
            if r.print {
                println!["{}", prog];
            }
        }
        Subcommand::Debug(r) => {
            let f = File::open(r.filename)?;
            let reader = io::BufReader::new(f);
            let prog = intcode::Program::new(reader);
            intcode::debugger::debug(prog)?;
        }
    };
    Ok(())
}
