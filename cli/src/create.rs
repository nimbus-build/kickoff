// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::Write,
    process::ExitCode,
    str::FromStr,
};

use clap::Args;

use kickoff::Manifest;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Target {
    name: String,
    runtime: &'static [u8],
}

impl FromStr for Target {
    type Err = String;

    fn from_str(target: &str) -> Result<Self, Self::Err> {
        match super::RUNTIMES.get(target) {
            Some(r) => Ok(Self {
                name: target.to_string(),
                runtime: r,
            }),
            None => Err(format!("unrecognized target: {}", target)),
        }
    }
}

impl Default for Target {
    fn default() -> Self {
        let cpu = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        let name = super::HOST_PLATFORMS
            .get(format!("{}-{}", cpu, os).as_str())
            .expect(format!("unrecognized host platform: {}-{}", cpu, os).as_str());

        let runtime = super::RUNTIMES
            .get(name)
            .expect(format!("runtime not found for platform: {}", name).as_str());

        Self {
            name: name.to_string(),
            runtime: runtime,
        }
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Create a self-contained launcher for an arbitrary executable.
#[derive(Debug, Args)]
pub struct Command {
    /// The manifest file.
    #[arg(long)]
    manifest: String,

    /// Write output to <OUTPUT>.
    #[arg(long)]
    output: String,

    /// The target platform triple.
    #[arg(long, required = false, value_parser = clap::value_parser!(Target), default_value_t)]
    target: Target,
}

impl Command {
    pub fn execute(&self) -> ExitCode {
        let manifest = match self.read_manifest() {
            Ok(m) => m,
            Err(err) => {
                eprintln!("[ERROR] Failed to read manifest file: {}", err);
                return ExitCode::from(1);
            }
        };

        match self.write_output(&manifest) {
            Ok(_) => ExitCode::from(0),
            Err(err) => {
                eprintln!("[ERROR] Failed to create launcher file: {}", err);
                ExitCode::from(1)
            }
        }
    }

    fn read_manifest(&self) -> Result<Manifest, std::io::Error> {
        let reader = File::open(&self.manifest)?;
        let manifest = serde_json::from_reader(reader)?;
        Ok(manifest)
    }

    #[cfg(unix)]
    fn write_output(&self, manifest: &Manifest) -> Result<(), std::io::Error> {
        use std::os::unix::fs::OpenOptionsExt;

        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o755)
            .open(&self.output)?;

        writer.write_all(self.target.runtime)?;
        kickoff::io::write_manifest(&mut writer, manifest)?;

        Ok(())
    }

    #[cfg(windows)]
    fn write_output(&self, manifest: &Manifest) -> Result<(), std::io::Error> {
        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.output)?;

        writer.write_all(self.target.runtime)?;
        kickoff::io::write_manifest(&mut writer, manifest)?;

        Ok(())
    }
}
