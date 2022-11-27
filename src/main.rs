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
#![allow(dead_code)]
use std::{env, io};
use std::io::{stderr, stdout, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct WalkingEntry {
    depth: usize,
    path: PathBuf,
    is_dir: bool,
    hidden: bool,
    line: String,
}

impl WalkingEntry {
    fn push_line(&mut self, s: &str) {
        self.line.push_str(s)
    }
}

struct Totals {
    dirs: i32,
    files: i32,
}

const SINGLE_ENTRY_STR: &str = "├── ";
const LAST_ENTRY_STR: &str = "└── ";
const PARENT_DELIM_STR: &str = "│   ";

fn main() {
    let str_vars = env::args().collect::<Vec<String>>();
    match parse_args(str_vars) {
        Ok(_) => std::process::exit(0),
        Err(e) => std::process::exit(e),
    };
}

fn emit_tree(root: &Path) -> Result<String, io::Error> {
    let mut entries: Vec<WalkingEntry> = Vec::new();
    let mut t = Totals { dirs: 0, files: 0 };
    let mut last_d = 0;
    let mut next_d = 0;
    let mut has_more = false;
    for entry in walkdir::WalkDir::new(root).sort_by_file_name() {
        let de = entry.as_ref()
            .unwrap();

        let hidden: bool = is_hidden(de.path());
        if hidden {
            continue
        }
        last_d = if de.depth() > 0 { de.depth() - 1 } else { 0 };
        next_d = de.depth();
        // Figure out the line to print for this entry
        let mut l = String::from(de.file_name().to_str().unwrap());
        let mut we = WalkingEntry {
            depth: de.depth(),
            path: de.path().to_path_buf().to_owned(),
            is_dir: de.path().is_dir(),
            hidden,
            line: l,
        };
        println!("Entry: {:?}", we);
        entries.push(we);
        if de.path().is_dir() && !de.path().eq(root) {
            t.dirs += 1;
        }
        if de.path().is_file() {
            t.files += 1;
        }
    };
    Ok(format!("{:?} directories, {:?} files\n", t.dirs, t.files))
}

#[cfg(target_os = "windows")]
fn is_hidden_file(p: &Path) -> bool {
    use std::fs::Metadata;
    let f = fs::metadata(p);
    match f {
        Ok(_) => {
            let a = f.file_attributes();
            return (a & 0x2) > 0;
        },
        Err(e) => {
            stderr().write_all(format!("Some error occurred, {}", e));
            return false;
        }
    }
}

#[cfg(target_os = "linux")]
fn is_hidden(p: &Path) -> bool {
    p.iter().map(|o| o.to_str())
        .map(|s| s.unwrap())
        .any(|s| s != "." && s.starts_with("."))
}

fn parse_args(args: Vec<String>) -> Result<(), i32>{
    let (mut a_flag, mut d_flag, mut l_flag) = (false, false, false);
    let args_i = args.iter();
    for arg in args_i.skip(1) {
        match arg.as_str() {
            "--help" => {
                usage();
                return Ok(());
            }
            "--version" => {
                version();
                return Ok(());
            }
            "-a" => a_flag = true,
            "-d" => d_flag = true,
            "-l" => l_flag = true,
            _ => {
                write_to_err(format!("tree-rs: Invalid argument `{}'.\n", arg));
                usage();
                return Err(1);
            }
        }
    }
    //TODO: Remove this stderr message. Using to suppress rustc warnings (for now)
    write_to_err(format!("Flags: a={a_flag} d={d_flag} l={l_flag}\n"));
    let out_s = emit_tree(Path::new("."))
        .or(Err(1));
    stdout().write_all(out_s?.as_bytes())
        .or(Err(1))
}

fn usage() {
    stdout()
        .write_all("usage: tree-rs [-adl] [--version] [--help] [--] [directory ...]\n".as_bytes())
        .unwrap();
}

fn version() {
    let ver = env!("CARGO_PKG_VERSION");
    stdout()
        .write_all(format!("tree v{}\n", ver).as_bytes())
        .unwrap();
}

fn write_to_err(content: String) {
    stderr().write_all(content.as_bytes())
        .expect("Failed to write err to stderr");
}


#[cfg(test)]
mod tests {

    #![allow(unused_imports)]

    use std::fs;

    use super::*;

    fn setup(tmpdir: &Path) {
        for i in vec![1, 2, 3, 4, 5].iter() {
            if i < &4 {
                fs::create_dir(tmpdir.join(format!("tmpd{i}")))
                    .expect("Unable to create temp dir");
                fs::File::create(tmpdir
                    .join(format!("tmpd{i}"))
                    .join("f")).expect("failed");

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

        let expected_s = "5 directories, 10 files.\n";
        assert!(emit_tree(d).is_ok());
        assert_eq!(emit_tree(d).unwrap(), expected_s);
    }

    #[test]
    fn hidden_file() {
        let tmpdir = tempfile::tempdir()
            .expect("could not create tmpdir");
        let p = tmpdir.path().join(".a");
        fs::File::create(p.as_path())
            .expect(format!("unable to create temp file {:?}", p.to_str().unwrap_or("")).as_str());
        assert!(is_hidden(Path::new(p.as_path())), "did not match")
    }

    #[test]
    fn file_within_hidden_dir() {
        let tmpdir = tempfile::tempdir()
            .expect("could not create tmpdir");
        fs::create_dir_all(tmpdir.path().join(".hidden").join("foo"))
            .expect("could not create hidden test dir");
        fs::File::create(tmpdir.path().join(".hidden").join("foo").join("bar"))
            .expect("could not create test file foo");
        assert!(is_hidden(tmpdir.path().join(".hidden").join("foo").join("bar").as_path()))
    }

    #[test]
    fn subdir_within_hidden_dir() {
        let tmpdir = tempfile::tempdir()
            .expect("could not create tmpdir");
        fs::create_dir_all(tmpdir.path().join(".hidden").join("foo"))
            .expect("could not create hidden test dir");
        assert!(is_hidden(tmpdir.path().join(".hidden").join("foo").as_path()))
    }
}
