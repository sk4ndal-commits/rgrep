use rgrep::{run, follow, ExitStatus};
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

    if cfg.follow {
        if let Err(err) = follow(&cfg, &inputs) {
            eprintln!("rgrep follow error: {}", err);
            return ExitCode::from(2);
        }
        // follow never returns on success; but if it returns, treat as success
        return ExitCode::from(0);
    }

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
