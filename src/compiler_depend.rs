//! A small module to parse CMake `compiler_depend.make` files

use regex::RegexBuilder;
use thiserror::Error;


#[derive(Debug, PartialEq, Eq)]
pub struct DepInfo<'h> {
    pub(crate) object: &'h str,
    pub(crate) deps: Vec<&'h str>
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected EOF while parsing file")]
    UnexpectedEOF,
    #[error("Line did not match expected pattern")]
    UnrecognizedLine,
    #[error("Unexpectedly encountered a colon. Filenames may not contain colons.")]
    UnexpectedColon,
}

// non-regex based parsing
pub(crate) fn extract_dependencies2(input: &str) -> Result<Vec<DepInfo>, ParseError> {
    let mut lines = input.lines();
    let mut dep_info = vec![];
    // `Some` if we have state due to line continuation on the previous iteration.
    let mut state: Option<DepInfo> = None;
    while let Some(line) = lines.next() {
        let mut finished_line_continuations: bool = false;
        match state {
            None => {
                // Check if line is a comment
                if line.starts_with('#') {
                    continue
                }
                if line.is_empty() {
                    continue
                }
                let (src, dep1) = line.split_once(':').ok_or(ParseError::UnrecognizedLine)?;
                if dep1.contains(':') {
                    return Err(ParseError::UnexpectedColon);
                }
                match dep1.strip_suffix('\\') {
                    Some(stripped) => {
                        let trimmed = stripped.trim();
                        let mut deps = vec![];
                        if !trimmed.is_empty() {
                            deps.push(trimmed);
                        }
                        state = Some(DepInfo {
                            object: src,
                            deps,
                        });
                    },
                    None => {
                        let trimmed = dep1.trim();
                        let mut deps = vec![];
                        if !trimmed.is_empty() {
                            deps.push(trimmed);
                        }
                        dep_info.push(DepInfo {
                            object: src,
                            deps,
                        })
                    }
                }
            },
            Some(ref mut info ) => {
                if line.contains(':') {
                    return Err(ParseError::UnexpectedColon);
                }
                let new_dep = match line.strip_suffix('\\') {
                    Some(stripped) => {
                        stripped.trim()
                    },
                    None => {
                        finished_line_continuations = true;
                        line.trim()
                    }
                };
                info.deps.push(new_dep);
            }
        }
        if finished_line_continuations {
            dep_info.push(state.expect("Expected to have DepInfo"));
            state = None;
        }
    }
    if state.is_some() {
        return Err(ParseError::UnexpectedEOF)
    }
    Ok(dep_info)
}

pub(crate) fn extract_dependencies(input: &str) -> Result<Vec<DepInfo>, ParseError> {
    let mut builder = RegexBuilder::new(r"^(?P<object_path>[^\s:\n#][^:\n]+):(?P<deps>((.*)\\\n)*?(.*))\n\n");
    builder.multi_line(true);
    let re = builder.build().expect("Invalid Regex");
    let mut v = Vec::new();
    for matches in re.captures_iter(input) {
        let obj = matches.name("object_path").expect("capture group not found");
        let deps_match = matches.name("deps").expect("capture group not found");
        let deps: Vec<&str> = deps_match.as_str().split(" \\\n").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        // eprintln!("obj {:?} has deps: {:?}", obj.as_str(), deps);
        let info = DepInfo {
            object: obj.as_str(),
            deps,
        };
        v.push(info);
    }
    Ok(v)
}


#[cfg(test)]
mod test {
    use crate::compiler_depend::DepInfo;

    const EXAMPLE_FILE : &str = r"# CMAKE generated file: DO NOT EDIT!

src/utils/CMakeFiles/utils.dir/utils.c.o: /home/jschwender/Dev/cmake_get_deps/tests/test_cmake_project/src/utils/utils.c \
  /home/jschwender/Dev/cmake_get_deps/tests/test_cmake_project/src/utils/include/test_project/utils/utils.h \
  /usr/include/bits/libc-header-start.h \
  /usr/lib/gcc/x86_64-redhat-linux/13/include/stdint.h

src/utils/CMakeFiles/utils.dir/blah.c.o: blah/blah.h

/usr/lib/gcc/x86_64-redhat-linux/13/include/stdint.h:

";

    #[test]
    fn extract_dependencies_test() {
        let depinfo = super::extract_dependencies(EXAMPLE_FILE).expect("Failed");
        let expected_utils = DepInfo {
            object: "src/utils/CMakeFiles/utils.dir/utils.c.o",
            deps: vec!["/home/jschwender/Dev/cmake_get_deps/tests/test_cmake_project/src/utils/utils.c",
                "/home/jschwender/Dev/cmake_get_deps/tests/test_cmake_project/src/utils/include/test_project/utils/utils.h",
                "/usr/include/bits/libc-header-start.h",
                "/usr/lib/gcc/x86_64-redhat-linux/13/include/stdint.h",
            ]
        };
        let expected_blah = DepInfo {
            object: "src/utils/CMakeFiles/utils.dir/blah.c.o",
            deps: vec!["blah/blah.h"],
        };
        let expected_sys = DepInfo {
            object: "/usr/lib/gcc/x86_64-redhat-linux/13/include/stdint.h",
            deps: vec![]
        };
        let expected = vec![expected_utils, expected_blah, expected_sys];
        assert_eq!(depinfo, expected);
    }

    #[test]
    fn extract_dependencies2_test() {
        let depinfo = super::extract_dependencies2(EXAMPLE_FILE).expect("Failed");
        let expected_utils = DepInfo {
            object: "src/utils/CMakeFiles/utils.dir/utils.c.o",
            deps: vec!["/home/jschwender/Dev/cmake_get_deps/tests/test_cmake_project/src/utils/utils.c",
                       "/home/jschwender/Dev/cmake_get_deps/tests/test_cmake_project/src/utils/include/test_project/utils/utils.h",
                       "/usr/include/bits/libc-header-start.h",
                       "/usr/lib/gcc/x86_64-redhat-linux/13/include/stdint.h",
            ]
        };
        let expected_blah = DepInfo {
            object: "src/utils/CMakeFiles/utils.dir/blah.c.o",
            deps: vec!["blah/blah.h"],
        };
        let expected_sys = DepInfo {
            object: "/usr/lib/gcc/x86_64-redhat-linux/13/include/stdint.h",
            deps: vec![]
        };
        let expected = vec![expected_utils, expected_blah, expected_sys];
        assert_eq!(depinfo, expected);
    }

}