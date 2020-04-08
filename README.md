# run-in-roblox
run-in-roblox is a tool to run a place, a model, or an individual script inside Roblox Studio.

run-in-roblox pipes output from inside Roblox Studio back to stdout/stderr, which enables traditional automation tools to work alongside Roblox.

## Installation

### With [Foreman](https://github.com/rojo-rbx/foreman)
run-in-roblox can be installed with Foreman, a toolchain manager for Roblox projects:

```toml
[tools]
remodel = { source = "rojo-rbx/remodel", version = "0.6.1" }
```

### From GitHub Releases
You can download pre-built binaries from [run-in-roblox's GitHub Releases page](https://github.com/rojo-rbx/remodel/releases).

### From crates.io
You'll need Rust 1.37.0 or newer.

```bash
cargo install remodel
```

## Usage
The recommended way to use `run-in-roblox` is with a place file and a script to run:

```bash
run-in-roblox MyPlace.rbxlx --script starter-script.lua
```

This will open `MyPlace.rbxlx` in Roblox Studio, run `starter-script.lua` until it completes, and then exit.

## License
run-in-roblox is available under the terms of the MIT License. See [LICENSE.txt](LICENSE.txt) or <https://opensource.org/licenses/MIT> for details.