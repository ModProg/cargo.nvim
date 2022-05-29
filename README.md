# cargo.nvim

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
