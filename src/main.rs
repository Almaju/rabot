use std::process::ExitCode;

fn main() -> ExitCode {
    rabot::cli::Cli::main(std::env::args_os())
}
