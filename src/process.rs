// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::io::Error as IOError;
use std::{collections::HashMap, ffi::OsString};

#[cfg(windows)]
pub fn execve(argv: &[OsString], env: &HashMap<OsString, OsString>) -> Result<(), IOError> {
    use std::os::windows::ffi::OsStrExt;
    use widestring::U16CString;

    // IMPORTANT: It may seem that some of the iteration pipelines below can be combined
    // together. However, this will not work because the allocations that happen within
    // the pipeline stages aren't guaranteed to outlive them. Thus, the allocated memory
    // may no longer be valid by the time we pass the pointers to "libc::execve(...)".
    //
    // TODO(alloveras): Investigate wether or not there is a better way to implement this.

    let argv = argv
        .iter()
        .map(|x| x.encode_wide().collect::<Vec<_>>())
        .map(|x| U16CString::from_vec(x).unwrap())
        .collect::<Vec<_>>();

    let env = env
        .iter()
        .map(|(k, v)| (k.encode_wide(), v.encode_wide()))
        .map(|(k, v)| (k.collect::<Vec<_>>(), v.collect::<Vec<_>>()))
        .map(|(k, v)| (k, OsString::from("=").encode_wide().collect(), v))
        .map(|(k, eq, v)| U16CString::from_vec([k, eq, v].concat()).unwrap())
        .collect::<Vec<_>>();

    let argv = argv
        .iter()
        .map(|x| x.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();

    let env = env
        .iter()
        .map(|x| x.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();

    unsafe { libc::wexecve(argv[0], argv.as_ptr(), env.as_ptr()) };

    Err(IOError::last_os_error())
}

#[cfg(unix)]
pub fn execve(argv: &[OsString], env: &HashMap<OsString, OsString>) -> Result<(), IOError> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};

    // IMPORTANT: It may seem that some of the iteration pipelines below can be combined
    // together. However, this will not work because the allocations that happen within
    // the pipeline stages aren't guaranteed to outlive them. Thus, the allocated memory
    // may no longer be valid by the time we pass the pointers to "libc::execve(...)".
    //
    // TODO(alloveras): Investigate wether or not there is a better way to implement this.

    let argv = argv
        .iter()
        .map(|x| x.as_bytes())
        .map(|x| CString::new(x).unwrap())
        .collect::<Vec<_>>();

    let env = env
        .iter()
        .map(|(k, v)| [k.as_bytes(), b"=", v.as_bytes()].concat())
        .map(|x| CString::new(x).unwrap())
        .collect::<Vec<_>>();

    let argv = argv
        .iter()
        .map(|x| x.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();

    let env = env
        .iter()
        .map(|x| x.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();

    unsafe { libc::execve(argv[0], argv.as_ptr(), env.as_ptr()) };

    Err(IOError::last_os_error())
}
