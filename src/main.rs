use std::{env, io};
use std::io::Write;



fn main() -> Result<(), io::Error> {
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--help" => {
                usage();
                break;
            },
            "--version" => {
                version();
                break;
            }
            _ => {
                write_to_err(&format!("Invalid argument `{arg}`"));
                usage();
                std::process::exit(2);
            }
        }
    }
    Ok(())
}

fn usage() {
    io::stdout()
        .write_all("\nusage: tree-rs [-ad] [--version] [--help] [--] [directory ...]\n".as_bytes())
        .unwrap();
}

fn version() {
    let ver = env!("CARGO_PKG_VERSION");
    io::stdout()
        .write_all(format!("\ntree v{}\n", ver).as_bytes())
        .unwrap();
}

fn write_to_err(content: &str) {
    io::stderr().write_all(content.as_bytes()).unwrap();
}
