use std::io::{self, BufRead};
use std::path::PathBuf;
use anyhow::Context;
use pico_args::Arguments;


/// Inputs:
/// --project-root /abs/path/to/project-root : Path to the CMake project root. If passed the output paths will be
///                                            relative to `project-root`, and non project paths are removed.
/// stdin: one path to a `compiler_depends.make` file per line. The user is expected to e.g. run `find` to get
///        a list of files and pipe the output into this program.
///
/// Output:
/// Print a sorted, deduplicated list of project-files, merged from the input `compiler_depends.make` files.
/// This can be used to e.g. generate the `rules:changes:paths` section of a gitlab-ci.yaml file.
fn main() -> Result<(), anyhow::Error>{
    let mut all_deps = vec![];
    let mut args = Arguments::from_env();
    let project_root: Option<PathBuf> = args.opt_value_from_str("--project-root")
        .context("Failed to parse argument")?;
    // Todo: Refactor to use path / pathbuf / osstr consistently
    let mut prefix = project_root.expect("--project-root is currently a required parameter").to_str().unwrap().to_string();
    prefix.push('/');
    // Simplistic hard-coded function to filter out
    // non user code.
    let filter_fn = |path: &str| -> Option<String> {
        // relative path: assume it is relative to the cwd for now.
        if !path.starts_with("/") {
            eprintln!("Relative path {:?} encountered...", path);
            return Some(path.into());
        }
        if let Some(stripped) = path.strip_prefix(&prefix) {
            Some(stripped.into())
        } else {
            None
        }
    };
    let stdin = io::stdin().lock();
    for arg in stdin.lines() {
        let filename = arg.unwrap();
        eprintln!("Parsing file: {filename}");
        let deps = cmake_get_deps::get_deps_from_cmake_depends_file(&filename, &filter_fn)?;
        all_deps.extend(deps);
    }
    eprintln!("Finished parsing all input files");
    all_deps.sort_unstable();
    all_deps.dedup();
    for dep in all_deps {
        println!("{dep}");
    }
    Ok(())
}