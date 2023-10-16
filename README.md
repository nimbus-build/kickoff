# Kickoff :hiking_boot:

[![GitHub](https://img.shields.io/github/license/nimbus-build/kickoff)](LICENSE)

A self-contained, unopinionated, fast and lightweight executable launcher.

## Supported Platforms

| Platform                    |        Host        |       Target       |
| :-------------------------- | :----------------: | :----------------: |
| aarch64-apple-macos-none    | :white_check_mark: | :white_check_mark: |
| aarch64-pc-windows-gnu [^1] |        :x:         | :white_check_mark: |
| aarch64-unknown-linux-gnu   | :white_check_mark: | :white_check_mark: |
| x86_64-apple-macos-none     | :white_check_mark: | :white_check_mark: |
| x86_64-pc-windows-gnu       |        :x:         | :white_check_mark: |
| x86_64-unknown-linux-gnu    | :white_check_mark: | :white_check_mark: |

[^1]: The Rust toolchain [does **not**](https://doc.rust-lang.org/nightly/rustc/platform-support.html) support the `aarch64-pc-windows-gnu` platform yet. However, `aarch64` devices running Windows 11 should be able to execute the `x86_64-pc-windows-gnu` binaries via [Windows on ARM (WoA) emulation](https://learn.microsoft.com/en-us/windows/arm/apps-on-arm-x86-emulation). Windows versions older than 11 are not supported on `aarch64` CPUs.

## Building from source

```
bazel build //...
bazel run //cli -- --help
```

## Usage Examples

**Unix (host == target)**

```shell
echo '{"argv": ["/bin/echo", "Hello World!"], "env": {}}' > manifest.json

kickoff create --manifest manifest.json --output hello-world

./hello-world
Hello World!

rm "manifest.json" "hello-world"
```

**Unix (host != target)**

On the host platform (e.g: `aarch64-apple-macos-none`):

```shell
echo {"argv": ["C:\\Windows\\System32\\cmd.exe", "/C", "echo Hello World!"], "env": {}} > manifest.json

kickoff create --manifest manifest.json --target x86_64-pc-windows-gnu --output hello-world.exe

rm "manifest.json" "hello-world.exe"
```

On the target platform `x86_64-pc-windows-gnu`:

```shell
hello-world.exe
Hello World!
```

## License

Copyright (c) 2023 The Nimbus Authors. All rights reserved.

The use of this source code is governed by a BSD-style
license that can be found in the LICENSE file.
