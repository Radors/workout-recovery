# Workout Recovery

`wr` is a CLI-tool for keeping track of physical recovery from working out.

## Installation

* Clone this repository
* Build with `cargo build --release`
* Move the compiled output `wr` (or `wr.exe` for windows), found in `./target/release/`, to a suitable directory.
* Add selected directory to PATH

## Usage

`wr --help` or `wr [COMMAND] --help`

There are only 3 commands: `list`, `add`, `remove`.

`wr add arbitrary`

`wr add "arbitrary description"`

`wr remove DM5G` (Review identifiers with `list` command)

`wr list` includes time elapsed since each added workout session, and this simple tracking of time is the main purpose of this tool.

## Notes

`wr` persists data by reading from and writing to a local JSON file on your machine.
The location of this file depends on your OS.

## Requirements:
* Rust (cargo, rustc) https://rustup.rs
* Tested on: Ubuntu 24 (although it should work on any common OS)
