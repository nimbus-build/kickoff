// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

mod substitutions;

use kickoff::Manifest;
use std::ffi::OsString;
use std::fs::File;
use std::io::Error as IOError;
use std::path::Path;
use std::process::ExitCode;

fn read_manifest(exe: &Path) -> Result<Manifest, IOError> {
    let mut file = File::open(&exe)?;
    kickoff::io::read_manifest(&mut file)
}

fn main() -> ExitCode {
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap().to_path_buf();

    let manifest = match read_manifest(&exe) {
        Ok(m) => m,
        Err(err) => {
            eprintln!(
                "[kickoff.runtime] Failed to read manifest from {}: {}",
                exe.to_str().unwrap_or("<unprintable>"),
                err
            );
            return ExitCode::from(1);
        }
    };

    let os_args = std::env::args_os().skip(1);
    let os_env = std::env::vars_os();

    let subs = vec![
        substitutions::fs::current_exe(&exe).unwrap(),
        substitutions::fs::current_dir(&dir).unwrap(),
    ];

    let manifest_args = manifest
        .argv
        .iter()
        .map(|a| OsString::from(a))
        .map(|a| substitutions::apply(&a, &subs));

    let manifest_env = manifest
        .env
        .iter()
        .map(|(k, v)| (OsString::from(k), OsString::from(v)))
        .map(|(k, v)| (k, substitutions::apply(&v, &subs)));

    let argv = manifest_args.chain(os_args).collect::<Vec<_>>();
    let env = os_env.chain(manifest_env).collect();

    let error = kickoff::process::execve(&argv, &env).unwrap_err();

    eprintln!(
        "[kickoff.runtime] Failed to call execve(\"{}\", [{}, ...], [...])",
        argv[0].to_str().unwrap_or("<unprintable>"),
        argv[0].to_str().unwrap_or("<unprintable>"),
    );

    ExitCode::from(error.raw_os_error().unwrap_or(1) as u8)
}
