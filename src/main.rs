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

struct Totals {
    dirs: usize,
    files: usize,
}

const SINGLE_ENTRY_STR: &str = "├── ";
const LAST_ENTRY_STR: &str = "└── ";
const PARENT_DELIM_STR: &str = "│   ";
const MAX_DEPTH_DELIM_STR: &str = "    ";

fn main() {
    let str_vars = env::args().collect::<Vec<String>>();
    match parse_args(str_vars) {
        Ok(_) => std::process::exit(0),
        Err(e) => std::process::exit(e),
    };
}

fn emit_tree(root: &Path) -> Result<String, io::Error> {
    let entries: Vec<WalkingEntry> = walkdir::WalkDir::new(root).sort_by_file_name().into_iter()
        .filter(|de| de.is_ok())
        .map(|de| de.unwrap())
        .map(|de| WalkingEntry {
                depth: de.depth(),
                path: de.path().to_path_buf().to_owned(),
                is_dir: de.path().is_dir(),
                hidden: is_hidden(de.path()),
                line: String::from(de.file_name().to_str().unwrap_or("")),
        })
        .collect();

    let mut lines: Vec<String> = Vec::new();
    let mut i = entries.iter()
        .filter(|e| !e.hidden)
        .into_iter()
        .peekable();

    let _max_d = entries.iter()
        .map(|e| e.depth)
        .max()
        .unwrap_or(0);
    let mut next_d: usize = 0;
    loop {
        let reached_end = i.peek().is_none();
        if !reached_end {
            next_d = i.peek().unwrap().depth
        }
        match i.next() {
            Some(e) => {
                let mut l = e.line.clone();
                if !e.path.as_path().eq(root) {
                    if next_d < e.depth {
                        l.insert_str(0, LAST_ENTRY_STR);
                    } else {
                        l.insert_str(0, SINGLE_ENTRY_STR);
                    }
                }
                for _ in 1..e.depth {
                    l.insert_str(0, PARENT_DELIM_STR)
                }
                l.push_str("\n");
                lines.push(l);
            }
            None => break
        }
    }

    let d = entries.iter()
        .filter(|&e| e.is_dir && !e.hidden && !e.path.eq(root))
        .count();
    let f = entries.iter()
        .filter(|&e| !e.is_dir && !e.hidden)
        .count();
    lines.push(format!("{:?} directories, {:?} files\n", d, f));
    Ok(lines.join(""))
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
    fn emit_tree_expected() {
        let tmpdir = tempfile::Builder::new().prefix("test").tempdir()
            .expect("should create tmpdir");
        let d = tmpdir.path();
        setup(d);

        let actual = emit_tree(d);
        if actual.is_ok() {
            println!("{}", actual.unwrap())
        }
        let expected_s = "5 directories, 10 files\n";
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
