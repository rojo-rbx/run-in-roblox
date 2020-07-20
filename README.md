# run-in-roblox
run-in-roblox is a tool to run a place, a model, or an individual script inside Roblox Studio.

run-in-roblox pipes output from inside Roblox Studio back to stdout/stderr, which enables traditional automation tools to work alongside Roblox.

## Installation

### From GitHub Releases
You can download pre-built binaries from [run-in-roblox's GitHub Releases page](https://github.com/rojo-rbx/run-in-roblox/releases).

### With [Foreman](https://github.com/rojo-rbx/foreman)
run-in-roblox can be installed with Foreman, a toolchain manager for Roblox projects.

```bash
[tools]
run-in-roblox = { source = "rojo-rbx/run-in-roblox", version = "0.3.0" }
```

### From crates.io
You'll need Rust 1.37.0 or newer.

```bash
cargo install run-in-roblox
```

## Usage
The recommended way to use `run-in-roblox` is with a place file and a script to run:

```bash
run-in-roblox --place MyPlace.rbxlx --script starter-script.lua
```

This will open `MyPlace.rbxlx` in Roblox Studio, run `starter-script.lua` until it completes, and then exit.

`--place` is optional, but `--script` is required.

## License
run-in-roblox is available under the terms of the MIT License. See [LICENSE.txt](LICENSE.txt) or <https://opensource.org/licenses/MIT> for details.