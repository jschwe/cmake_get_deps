use anyhow::Context;
use crate::compiler_depend::{DepInfo, ParseError};

pub mod compiler_depend;

pub fn get_deps_from_cmake_depends_file<F>(path: &str, f: &F) -> Result<Vec<String>, anyhow::Error>
where F: Fn(&str) -> Option<String>{
    assert!(path.ends_with("compiler_depend.make"));
    let dep_file = std::fs::read_to_string(path)
        .context("Failed to read compile_depends.make file")?;
    let deps = compiler_depend::extract_dependencies(&dep_file)?;
    let mut all_deps_filtered = vec![];
    for dep in deps {
        let filtered = dep.deps.into_iter()
            .filter_map(f);
        all_deps_filtered.extend(filtered);
    }
    Ok(all_deps_filtered)
}