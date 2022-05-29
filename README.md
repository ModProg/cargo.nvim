# cargo.nvim

## Installation
### `packer.nvim`
depends on `cargo-make`:

```lua
use {
    'ModProg/cargo.nvim', 
    run = 'makers release',
    config = function()
        require"cargo".setup()
    end
}
```

### Manually

1. Compile the code with `cargo build --release`
2. Copy `target/release/libcargo_nvim.so` to your nvim directory `lua/cargo.so`

## Commands
- `:Cargo add ...` adds a dependency + reloads workspace (depends on `cargo-edit`)
- `:Cargo rm dep` removes a dependency + reloads workspace (depends on `cargo-edit`)
- `:Cargo reload` reloads workspace
- `:Cargo ...` runs any other cargo command in a terminal
