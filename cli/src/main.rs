// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::process::ExitCode;

use clap::{command, Parser, Subcommand};
use phf::phf_map;

mod create;

static HOST_PLATFORMS: phf::Map<&'static str, &'static str> = phf_map! {
    "aarch64-linux" => "aarch64-unknown-linux-gnu",
    "aarch64-macos" => "aarch64-apple-macos-none",
    "aarch64-windows" => "aarch64-pc-windows-gnu",
    "x86_64-linux" => "x86_64-unknown-linux-gnu",
    "x86_64-macos" => "x86_64-apple-macos-none",
    "x86_64-windows" => "x86_64-pc-windows-gnu",
};

static RUNTIMES: phf::Map<&'static str, &'static [u8]> = phf_map! {
    "aarch64-apple-macos-none" => include_bytes!(concat!("../../",env!("BAZEL_GENDIR"), "/runtime/kickoff-runtime-aarch64-macos")),
    // For now, the "aarch64-pc-windows-gnu" target uses the "x86_64-pc-windows-gnu" runtime binary
    // because the Rust toolchain cannot produce binaries for the former platform. This works because
    // on aarch64 devices, Windows 11 can run x86_64 binaries via emulation.
    "aarch64-pc-windows-gnu" => include_bytes!(concat!("../../", env!("BAZEL_GENDIR"), "/runtime/kickoff-runtime-x86_64-windows.exe")),
    "aarch64-unknown-linux-gnu" => include_bytes!(concat!("../../", env!("BAZEL_GENDIR"), "/runtime/kickoff-runtime-aarch64-linux")),
    "x86_64-apple-macos-none" => include_bytes!(concat!("../../", env!("BAZEL_GENDIR"), "/runtime/kickoff-runtime-x86_64-macos")),
    "x86_64-pc-windows-gnu" => include_bytes!(concat!("../../", env!("BAZEL_GENDIR"), "/runtime/kickoff-runtime-x86_64-windows.exe")),
    "x86_64-unknown-linux-gnu" => include_bytes!(concat!("../../", env!("BAZEL_GENDIR"), "/runtime/kickoff-runtime-x86_64-linux")),
};

#[derive(Debug, Parser)]
#[command(name = "kickoff", author, version, about, long_about = None )]
struct Kickoff {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Create(create::Command),
}

fn main() -> ExitCode {
    let args = Kickoff::parse();

    match args.cmd {
        Commands::Create(cmd) => cmd.execute(),
    }
}
