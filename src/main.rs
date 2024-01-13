use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use anyhow::Context;
use pico_args::Arguments;

// This is a really hacked together MVP and could use some more thought.
fn limit_to_n_with_wildcards(mut file_paths: Vec<PathBuf>, n: usize) -> Vec<PathBuf> {
    file_paths.sort_unstable();
    file_paths.dedup();
    if file_paths.len() <= n {
        return file_paths;
    }
    let mut map: HashMap<&Path, Vec<&PathBuf>> = HashMap::new();
    let mut new_merged = vec![];
    let mut un_merged = vec![];
    let file_paths = file_paths;
    for f in &file_paths {
        let parent = f.parent().expect("not a file?");
        if let Some(vec) = map.get_mut(parent) {
            vec.push(f);
        } else {
            map.insert(parent, vec![f]);
        }
    }

    for (dir, dep_files) in map {
        let extension = dep_files.first().unwrap().extension();
        if dep_files.iter().all(|f| f.extension() == extension) {
            let dir_entries = dir.read_dir().expect("Read dir failed");
            let perfect_wildcard_candidate = dir_entries.filter_map(|entry| entry.ok().map(|e| e.path().extension() == extension))
                .all(|same_extension| same_extension);
            if perfect_wildcard_candidate {
                let mut file_name = OsString::from("*");
                if let Some(extension) = extension {
                    file_name.push(".");
                    file_name.push(extension)
                }
                new_merged.push(dir.join(file_name))
            } else {
                un_merged.extend(dep_files)
            }
        }
    }
    let new_len =new_merged.len() + un_merged.len();
    if new_len > n {
        // We could try to merge more by not requiring that the wildcard matches the original results exactly.
        eprintln!("After merge we still have {new_len} entries");
    }
    new_merged.extend(un_merged.iter().map(|&item| item.to_owned()));
    new_merged
}


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
    // Attempt to reduce the amount of paths with wildcards to below the passed number.
    let opt_wildcard_merge_limit: Option<usize> = args.opt_value_from_str("--reduce-with-wildcards")
        .context("Invalid Value passed")?;
    let relative_base_dir: PathBuf = args.opt_value_from_str("--relative-to")?
        .expect("--relative-to is currently a required parameter");
    // Todo: Refactor to use path / pathbuf / osstr consistently
    let mut prefix = project_root.expect("--project-root is currently a required parameter").to_str().unwrap().to_string();
    prefix.push('/');
    // Simplistic hard-coded function to filter out
    // non user code.
    let filter_fn = |path: &str| -> Option<String> {
        // relative path: assume it is relative to the cwd for now.
        if !path.starts_with("/") {
            let rel_path = PathBuf::from(path);
            assert!(rel_path.is_relative());
            let abs_path = relative_base_dir.join(rel_path);
            let abs_path = abs_path.canonicalize().expect("Failed to canonicalize");
            assert!(abs_path.is_file());
            let stripped = abs_path.strip_prefix(&prefix).expect("Relative path could not be resolved inside project-root");
            return Some(stripped.to_str().unwrap().to_string());
        }
        if let Some(stripped) = path.strip_prefix(&prefix) {
            Some(stripped.into())
        } else {
            None
        }
    };
    let stdin = io::stdin().lock();
    let mut count = 0;
    for arg in stdin.lines() {
        let filename = arg.unwrap();
        eprintln!("Parsing file: {filename}");
        let deps = cmake_get_deps::get_deps_from_cmake_depends_file(&filename, &filter_fn)?;
        all_deps.extend(deps);
        count += 1;
    }
    if count == 0 {
        anyhow::bail!("No input files received on stdin")
    }
    eprintln!("Finished parsing all input files");
    all_deps.sort_unstable();
    all_deps.dedup();
    eprintln!("Depends on {} files", all_deps.len());
    if let Some(limit) = opt_wildcard_merge_limit {
        let p_deps = all_deps.iter().map(PathBuf::from).collect();
        let merged = limit_to_n_with_wildcards(p_deps, limit);
        all_deps = merged.iter().map(|p| p.to_str().unwrap().to_string()).collect();
        eprintln!("Reduced path count to {}", all_deps.len());
    }
    for dep in all_deps {
        println!("{dep:?}");
    }
    Ok(())
}
