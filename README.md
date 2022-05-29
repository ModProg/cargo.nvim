# cargo.nvim

## Installation
To install with `packer.nvim` (depends on `cargo-make`):

```lua
use {
    'ModProg/cargo.nvim', 
    run = 'makers release',
    config = function()
        require"cargo".setup()
    end
}
```

## Commands
- `:Cargo add ...` adds a dependency + reloads workspace (depends on `cargo-edit`)
- `:Cargo rm dep` removes a dependency + reloads workspace (depends on `cargo-edit`)
- `:Cargo reload` reloads workspace
- `:Cargo ...` runs any other cargo command in a terminal
