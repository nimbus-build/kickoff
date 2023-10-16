// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::ffi::{OsStr, OsString};

pub mod fs;

type SubstitutionFn<'a> = dyn Fn(&str) -> String + 'a;

pub fn apply<T>(input: &OsStr, substitutions: &[T]) -> OsString
where
    T: Fn(&str) -> String,
{
    let input_str = match input.to_str() {
        Some(x) => x,
        None => return input.to_owned(),
    };

    let mut result = String::from(input_str);
    for substitution in substitutions {
        result = substitution(&result)
    }

    OsString::from(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::ffi::OsString;

    #[test]
    #[cfg(unix)]
    fn apply_when_input_not_utf8_then_noop() {
        use std::os::unix::prelude::OsStringExt;

        let input = OsString::from_vec(vec![0xC3, 0x28]); // Invalid UTF-8 sequence.
        let subs = vec![|_: &str| -> String { String::new() }];

        assert_eq!(apply(&input, &subs), input);
    }

    #[test]
    #[cfg(windows)]
    fn apply_when_input_not_utf16_then_noop() {
        use std::os::windows::ffi::OsStringExt;

        let input = OsString::from_wide(vec![0x0066, 0x006F, 0xD800, 0x006F]); // Invalid UTF-16 sequence.
        let subs = vec![|_: &str| -> String { String::new() }];

        assert_eq!(apply(&input, &subs), input);
    }

    #[test]
    fn apply_when_substitutions_not_match_then_noop() {
        let input = OsString::from("some {var} input");

        let subs = vec![
            |x: &str| -> String {
                let re = Regex::new(r"non-matching-one").unwrap();
                re.replace_all(x, "interpolated").to_string()
            },
            |x: &str| -> String {
                let re = Regex::new(r"non-matching-two").unwrap();
                re.replace_all(x, "interpolated").to_string()
            },
        ];

        assert_eq!(apply(&input, &subs), "some {var} input")
    }

    #[test]
    fn apply_when_substitutions_match_then_substitutes() {
        let input = OsString::from("both {some} and {other} are interpolated");

        let subs = vec![
            |x: &str| -> String {
                let re = Regex::new(r"\{some\}").unwrap();
                re.replace_all(x, "first-value").to_string()
            },
            |x: &str| -> String {
                let re = Regex::new(r"\{other\}").unwrap();
                re.replace_all(x, "second-value").to_string()
            },
            |x: &str| -> String {
                let re = Regex::new(r"\{non-matching\}").unwrap();
                re.replace_all(x, "third-value").to_string()
            },
        ];

        assert_eq!(
            apply(&input, &subs),
            "both first-value and second-value are interpolated"
        )
    }
}
