use std::{env, io};
use std::io::Write;


fn main() {
    let env_vars = env::args();
    let str_vars = env_vars.map(|a| {
        a
    }).collect::<Vec<String>>();
    match parse_args(str_vars) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            write_to_err(&e);
            std::process::exit(2);
        }
    };
}

fn parse_args(args: Vec<String>) -> Result<(), String>{
    let args_i = args.iter();
    for arg in args_i.skip(1) {
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
                usage();
                return Err(format!("Invalid argument `{arg}`").to_string());
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


#[cfg(test)]
mod tests {

    #![allow(unused_imports)]

    use super::*;

    #[test]
    fn parse_empty_args() {
        let test_args = vec!["name/of/program".to_string()];
        assert!(parse_args(test_args).is_ok())
    }

    #[test]
    fn bad_arg() {
        let test_args = vec!["name/of/program".to_string(), "--foo".to_string()];
        assert!(parse_args(test_args).is_err())
    }
}
