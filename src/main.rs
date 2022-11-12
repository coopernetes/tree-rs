/* Copyright (c) 2022 by Thomas Cooper
  All Rights reserved

  This program is free software; you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation; either version 2 of the License, or
  (at your option) any later version.

  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program; if not, write to the Free Software
  Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
*/
use std::{env, fs, io};
use std::io::{stdout, stderr, Write};
use std::path::Path;

use walkdir;

fn main() {
    let env_vars = env::args();
    let str_vars = env_vars.collect::<Vec<String>>();
    match parse_args(str_vars) {
        Ok(_) => std::process::exit(0),
        Err(e) => std::process::exit(e),
    };
}

fn create_fd_count(p: &Path) -> Result<String, io::Error> {
    let (mut d, mut f) = (0, 0);
    for entry in walkdir::WalkDir::new(p).into_iter().skip(1) {
        let de = entry.as_ref()
            .unwrap();

        // TODO: Fix this filtering. If any parent dirs start with a '.', skip is true.
        //       By default, TMPDIR = /tmp/.tmp<randomBits> and every DirEntry is skipped.
        let skip = de.path().iter().any(|s| {
            let c = s.to_str().unwrap().chars().next().unwrap();
            c == '.'
        });
        if skip {
            continue
        }
        if de.path().is_dir() {
            d += 1;
        }
        if de.path().is_file() {
            f += 1;
        }
    };
    Ok(format!("{d} directories, {f} files."))
}

fn parse_args(args: Vec<String>) -> Result<(), i32>{
    let (mut a_flag, mut d_flag) = (false, false);
    let args_i = args.iter();
    for arg in args_i.skip(1) {
        match arg.as_str() {
            "--help" => {
                io::stdout().write_all("\n".as_bytes()).unwrap();
                usage();
                return Ok(())
            }
            "--version" => {
                version();
                return Ok(())
            }
            "-a" => a_flag = true,
            "-d" => d_flag = true,
            _ => {
                write_to_err(format!("tree-rs: Invalid argument `{}'.\n", arg));
                usage();
                return Err(1);
            }
        }
    }
    let out_s = create_fd_count(Path::new("."))
        .or(Err(1));
    stdout().write_all(out_s?.as_bytes())
        .or(Err(1))
}

fn usage() {
    io::stdout()
        .write_all("usage: tree-rs [-ad] [--version] [--help] [--] [directory ...]\n".as_bytes())
        .unwrap();
}

fn version() {
    let ver = env!("CARGO_PKG_VERSION");
    stdout()
        .write_all(format!("\ntree v{}\n", ver).as_bytes())
        .unwrap();
}

fn write_to_err(content: String) {
    stderr().write_all(content.as_bytes())
        .expect("Failed to write err to stderr");
}


#[cfg(test)]
mod tests {

    #![allow(unused_imports)]

    use tempfile;

    use super::*;

    fn setup(tmpdir: &Path) {
        for i in vec![1, 2, 3, 4, 5].iter() {
            if i < &4 {
                fs::create_dir(tmpdir.join(format!("tmpd{i}")))
                    .expect("Unable to create temp dir");
                fs::File::create(tmpdir
                    .join(format!("tmpd{i}"))
                    .join(format!("f"))).expect("failed");

            }
            fs::File::create(tmpdir.join(format!("tmpf{i}")))
                .expect("Unable to create temp file");
        }
        fs::create_dir(tmpdir.join("foo"))
            .expect("failed to create foo/");
        fs::create_dir(tmpdir.join("foo").join("bar"))
            .expect("failed to create foo/bar");
        fs::File::create(tmpdir.join("foo").join("bar").join("baz"))
            .expect("failed");
        fs::File::create(tmpdir.join("foo").join("bar").join("qux"))
            .expect("failed");
        fs::File::create(tmpdir.join("foo").join("bar").join(".hidden"))
            .expect("failed");
    }

    #[test]
    fn parse_empty_args() {
        let test_args = vec!["name/of/program".to_string()];
        assert!(parse_args(test_args).is_ok())
    }

    #[test]
    fn parse_bad_arg() {
        let test_args = vec!["name/of/program".to_string(), "--foo".to_string()];
        assert!(parse_args(test_args).is_err());

        let test_args = vec!["".to_string(), "--bar".to_string()];
        assert!(parse_args(test_args).is_err())
    }

    #[test]
    fn parse_good_args() {
        let valid_args = vec![
            "--help",
            "--version",
        ];
        for a in valid_args {
            let test_args = vec!["name/of/program".to_string(), a.to_string()];
            assert!(parse_args(test_args).is_ok())
        }
    }

    #[test]
    fn create_fd_count_expected() {
        let tmpdir = tempfile::tempdir()
            .expect("should create tmpdir");
        let d = tmpdir.path();
        setup(d);

        let expected_s = "5 directories, 10 files.";
        assert!(create_fd_count(d).is_ok());
        assert_eq!(create_fd_count(d).unwrap(), expected_s);
    }
}
