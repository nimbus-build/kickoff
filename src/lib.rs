// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod io;
pub mod process;

#[derive(Default, Debug)]
pub struct Section {
    pos: u64,
    len: u64,
}

#[derive(Default, Debug)]
pub struct Trailer {
    magic: [u8; 8],
    runtime: Section,
    manifest: Section,
}

#[derive(PartialEq, Default, Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub argv: Vec<String>,
    pub env: HashMap<String, String>,
}
