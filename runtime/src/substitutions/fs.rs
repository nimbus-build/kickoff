// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use regex::Regex;
use std::{error::Error, path::Path};

use super::SubstitutionFn;

pub fn current_exe<'a>(path: &'a Path) -> Result<Box<SubstitutionFn<'a>>, Box<dyn Error>> {
    let re = Regex::new(r"\{kickoff.self.path\}")?;

    let path = path
        .to_str()
        .ok_or("the current executable path is not convertible to UTF-8")?;

    let closure = move |x: &str| -> String { re.replace_all(x, path).to_string() };

    Ok(Box::new(closure))
}

pub fn current_dir<'a>(path: &'a Path) -> Result<Box<SubstitutionFn<'a>>, Box<dyn Error>> {
    let re = Regex::new(r"\{kickoff.self.dir\}")?;

    let path = path
        .to_str()
        .ok_or("the current executable directory is not convertible to UTF-8")?;

    let closure = move |x: &str| -> String { re.replace_all(x, path).to_string() };

    Ok(Box::new(closure))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn current_exe_when_input_contains_pattern_then_substitutes() {
        let path = PathBuf::from("/some/path");
        let func = current_exe(&path).unwrap();

        assert_eq!(func("{kickoff.self.path}"), "/some/path");

        assert_eq!(
            func("the path is {kickoff.self.path}"),
            "the path is /some/path"
        );

        assert_eq!(
            func("{do.not.match} {kickoff.self.path}"),
            "{do.not.match} /some/path"
        );

        assert_eq!(
            func("one: {kickoff.self.path} two: {kickoff.self.path}"),
            "one: /some/path two: /some/path"
        )
    }

    #[test]
    fn current_exe_when_input_not_contains_pattern_then_noop() {
        let path = PathBuf::from("/some/path");
        let func = current_exe(&path).unwrap();

        assert_eq!(func("does not contain pattern"), "does not contain pattern");
        assert_eq!(func("{do.not.match}"), "{do.not.match}");
    }

    #[test]
    fn current_dir_when_input_contains_pattern_then_substitutes() {
        let path = PathBuf::from("/some/path");
        let func = current_dir(&path).unwrap();

        assert_eq!(func("{kickoff.self.dir}"), "/some/path");

        assert_eq!(
            func("the path is {kickoff.self.dir}"),
            "the path is /some/path"
        );

        assert_eq!(
            func("{do.not.match} {kickoff.self.dir}"),
            "{do.not.match} /some/path"
        );

        assert_eq!(
            func("one: {kickoff.self.dir} two: {kickoff.self.dir}"),
            "one: /some/path two: /some/path"
        )
    }

    #[test]
    fn current_dir_when_input_not_contains_pattern_then_noop() {
        let path = PathBuf::from("/some/path");
        let func = current_dir(&path).unwrap();

        assert_eq!(func("does not contain pattern"), "does not contain pattern");
        assert_eq!(func("{do.not.match}"), "{do.not.match}");
    }
}
