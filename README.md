# tree-rs
A rewrite of [tree](https://github.com/Old-Man-Programmer/tree) in Rust.

Restrictions:
- no modifications to original C code
- avoid using any external crates unless absolutely necessary
- don't blindly replicate or copy the original C code but take inspiration (use Rust types)

## 0.1.0
- [x] read arguments from the terminal when called
- [x] print matching (roughly, most flags won't be supported) usage when `--help` is used
- [x] return exit code 0
- [x] print version string (bonus: take from Cargo.toml)
- [x] handle unsupported argument. Print `Invalid argument \`--arg\`` & usage and return exit code 2
- [x] write a rust test to confirm the behaviour

## 0.1.1
- [ ] when called with no args, print `X directories, Y files`

## 0.1.2
- [ ] display tree view of directories & files in current working dir
- [ ] support `-a` option (show all files including hidden)
- [ ] support `-d` option (list directories only)
- [ ] if any directory cannot be opened (permission, etc), return exit code 2. Else return 0 on success

## 0.1.3
- [ ] support arbitrary list of directories as positional args and perform tree search & display
- [ ] when multiple directories are used, the count of dirs & files includes all locations
- [ ] write integration test, assert C `tree` == `tree-rs`
