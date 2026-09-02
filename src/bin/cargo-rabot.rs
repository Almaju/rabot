//! `cargo rabot check`: cargo passes `rabot` as the first argument, which is
//! dropped so the rest reads exactly like the standalone binary.

use std::ffi::OsString;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    if args.get(1).is_some_and(|arg| arg == "rabot") {
        args.remove(1);
    }
    rabot::cli::Cli::main(args)
}
