//! A small module to parse CMake `compiler_depend.make` files

use regex::RegexBuilder;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DepInfo<'h> {
    object: &'h str,
    deps: Vec<&'h str>
}

#[derive(Debug)]
pub(crate) enum ParseError {

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
        println!("obj {:?} has deps: {:?}", obj.as_str(), deps);
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

}