use std::io::{self, BufRead};



fn main() -> Result<(), anyhow::Error>{
    let mut all_deps = vec![];
    let cwd = std::env::current_dir()?;
    // Todo: Refactor to use path / pathbuf / osstr consistently
    let mut prefix = cwd.to_str().unwrap().to_string();
    prefix.push('/');
    // Simplisitic hard-coded function to filter out
    // non user code.
    let filter_fn = |path: &str| -> Option<String> {
        // relative path: assume it is relative to the cwd for now.
        if !path.starts_with("/") {
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
    for dep in all_deps {
        println!("{dep}");
    }
    Ok(())
}