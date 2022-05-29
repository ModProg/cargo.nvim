use std::{
    ffi::OsStr,
    iter,
    process::{Command, Output},
};

use async_process::Command as AsyncCommand;
use clap::{ErrorKind, Parser};
use mlua::prelude::*;
use nvim::{
    api::{
        Api, Callback, Command as VimCommand, CommandCallbackData, Nargs, NvimCreateAutocmdsOpts,
        NvimCreateUserCommandOpts,
    },
    common::Buffer,
    lsp::{Handler, Lsp},
    LogLevel, Vim,
};

#[derive(Parser, Debug)]
#[clap(
    no_binary_name = true,
    disable_version_flag = true,
    disable_help_flag = true,
    bin_name = ":Cargo",
    name = "Cargo"
)]
enum Cli {
    #[clap(allow_hyphen_values = true)]
    Add {
        args: Vec<String>,
    },
    Reload,
    #[clap(alias = "rm")]
    Remove {
        #[clap(name = "crate")]
        crate_: String,
    },
    #[clap(external_subcommand)]
    Other(Vec<String>),
}

fn ensure_cargo_edit(command: &str, vim: &Vim) -> bool {
    if let Ok(output) = Command::new("cargo").args(["help", command]).output() {
        if output.status.success() {
            true
        } else {
            vim.notify(
                "command `cargo rm` is not availible, make sure to install `cargo-edit`",
                LogLevel::Error,
                None,
            );
            false
        }
    } else {
        vim.notify(
            "command `cargo` is not availible, make sure to install `cargo`",
            LogLevel::Error,
            None,
        );
        false
    }
}

async fn cargo_command<I, S>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let Output { status, stderr, .. } = AsyncCommand::new("cargo")
        .args(args)
        .output()
        .await
        .expect("Cargo is installed");
    let stderr = String::from_utf8(stderr).expect("cargo outputs utf8");
    if status.success() {
        Ok(stderr)
    } else {
        Err(stderr)
    }
}

fn cargo_reload(lua: &Lua) {
    Lsp::from(lua).buf_request(
        Buffer::Current,
        "rust-analyzer/reloadWorkspace".into(),
        None,
        Handler::None,
    );
}

fn setup(lua: &Lua, _config: Option<LuaTable>) -> LuaResult<()> {
    Api::from(lua).nvim_create_autocmd(
        vec!["FileType".to_owned()],
        NvimCreateAutocmdsOpts::builder()
            .pattern(vec!["rust".to_owned()])
            .callback(Callback::from_fn(lua, |lua, data| {
                Api::from(lua).nvim_buf_create_user_command(
                    data.buf.into(),
                    "Cargo".into(),
                    VimCommand::from_fn_async(
                        lua,
                        |lua, CommandCallbackData { fargs, .. }| async move {
                            let vim = Vim::from(lua);
                            let api = Api::from(lua);
                            match Cli::try_parse_from(fargs) {
                                Ok(Cli::Reload) => cargo_reload(lua),
                                Ok(Cli::Remove { crate_ }) => {
                                    if ensure_cargo_edit("rm", &vim) {
                                        match cargo_command(["rm", &crate_]).await {
                                            Ok(output) => {
                                                cargo_reload(lua);
                                                vim.notify(output.trim(), LogLevel::Info, None)
                                            }
                                            Err(output) => vim.notify(
                                                output.lines().last().unwrap_or_default().trim(),
                                                LogLevel::Error,
                                                None,
                                            ),
                                        }
                                    }
                                }
                                Ok(Cli::Add { args }) => {
                                    if ensure_cargo_edit("add", &vim) {
                                        match cargo_command(
                                            iter::once("add".to_owned()).chain(args.into_iter()),
                                        )
                                        .await
                                        {
                                            Ok(output) => {
                                                let mut warnings = String::new();
                                                let mut addings = String::new();
                                                for line in output.lines() {
                                                    let line = line.trim();
                                                    if line.starts_with("Warning") {
                                                        warnings += &format!("\n{line}");
                                                    }
                                                    if line.starts_with("Adding")
                                                        || !addings.is_empty()
                                                    {
                                                        addings += &format!("{line} ")
                                                    }
                                                }
                                                if !warnings.is_empty() {
                                                    vim.notify(&warnings, LogLevel::Warn, None);
                                                }
                                                cargo_reload(lua);
                                                vim.notify(addings.trim_end(), LogLevel::Info, None)
                                            }
                                            Err(output) => vim.notify(
                                                output.lines().last().unwrap_or_default().trim(),
                                                LogLevel::Error,
                                                None,
                                            ),
                                        }
                                    }
                                }
                                Ok(Cli::Other(args)) => {
                                    api.nvim_exec(
                                        &format!(
                                            "execute 'noautocmd new | terminal cargo {}'",
                                            args.join(" ")
                                                .replace('\\', "\\")
                                                .replace('\'', "\\\'")
                                        ),
                                        false,
                                    );
                                }
                                Err(err) if err.kind() == ErrorKind::DisplayHelp => {
                                    vim.notify(&format!("{err}"), LogLevel::Info, None)
                                }
                                Err(err) => vim.notify(&format!("{err}"), LogLevel::Error, None),
                            }
                        },
                    ),
                    NvimCreateUserCommandOpts::builder()
                        .nargs(Nargs::OneOrMore)
                        .build()
                        .unwrap(),
                );
                false
            }))
            .build()
            .unwrap(),
    );
    Ok(())
}

#[mlua::lua_module]
fn cargo(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("setup", lua.create_function(setup)?)?;
    Ok(exports)
}
