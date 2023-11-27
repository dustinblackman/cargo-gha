<h1 align=center>cargo-gha</h1>

![cargo-gha](.github/banner.png)

> Version lock, cache, and run binaries from any Github Release assets. Pull in external tools and keep the versions in sync across your team, and forget installing globally.

- [Overview](#overview)
- [Install](#install)
- [Usage](#usage)
  - [cargo gha BINARY](#cargo-gha-crate)
  - [cargo gha --install](#cargo-gha---install)
- [License](#license)

## Overview

**Notice:** This is an early release as the mechanics of the tool are fleshed out. Tests have yet to be completed, changes are expected.

A companion tool to [`cargo-run-bin`](https://github.com/dustinblackman/cargo-run-bin), `cargo-gha` handles downloading
and persisting tools from any Github Release assets, keeping versions available and in sync across your team.

[![asciicast](https://asciinema.org/a/604129.svg)](https://asciinema.org/a/604129)

## Install

Run the following to install `cargo-gha`, and ignore the cache directory in your project.

```sh
cargo install cargo-gha
echo ".gha/" >> .gitignore
```

Or if using [`cargo-run-bin`](https://github.com/dustinblackman/cargo-run-bin), add it Cargo.toml.

```toml
[package.metadata.bin]
cargo-gha = { version = "0.4.6" }
```

```sh
cargo bin --sync-aliases
echo ".gha/" >> .gitignore
```

## Usage

`cargo-gha` has an interactive experience to add assets to Cargo.toml. Before starting, you must specify the target architectures you and your team use.

```toml
[package.metadata.gha]
targets = ["x86_64-apple-darwin", "x86_64-unknown-linux-gnu"]
```

Once set, run the following to add an asset and follow the steps. As an example, let's add ProtocolBuffer's `protoc`.

```sh
cargo gha --add protocolbuffers/protobuf
# Or
cargo gha --add https://github.com/protocolbuffers/protobuf
```

Installed! `protoc` is now available through `cargo-gha`. Try it!

```sh
cargo gha protoc --help
```

### `cargo gha BINARY`

Taking an example of `protoc`, running `cargo gha protoc --help` with install and cache the protoc binary with the
specified version in `Cargo.toml`. All future executions will run instantly without an install step, and protoc can be used
as you wish!

### `cargo gha --install`

When pulling down a new repo, or adding a step to CI, `cargo gha --install` will install all assets that have not been
cached which are configured in `Cargo.toml`.

## [License](./LICENSE)

MIT.
