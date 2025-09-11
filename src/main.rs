use rgrep::{run, ExitStatus};
use std::process::ExitCode;

mod cli;

fn main() -> ExitCode {
    let (cfg, inputs) = match cli::parse() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{}", msg);
            return ExitCode::from(2);
        }
    };

    match run(&cfg, &inputs) {
        Ok(result) => {
            if !cfg.quiet {
                print!("{}", result.output);
            }
            match result.status {
                ExitStatus::MatchFound => ExitCode::from(0),
                ExitStatus::NoMatch => ExitCode::from(1),
            }
        }
        Err(err) => {
            eprintln!("rgrep error: {}", err);
            ExitCode::from(2)
        }
    }
}
