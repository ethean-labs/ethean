//! After every `cargo build` of this package, publish a PATH shim into
//! `$CARGO_HOME/bin` so the command `ethean` resolves on Windows and Unix.
//!
//! The shim prefers `target/release/ethean`, then `target/debug/ethean`.
//! No separate install.sh is required.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_HOME");

    if let Err(err) = publish_path_shim() {
        println!("cargo:warning=ethean PATH shim not updated: {err}");
    }
}

fn publish_path_shim() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(to_io)?);
    // bin/ethean -> workspace root
    let workspace = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "workspace root"))?
        .to_path_buf();

    let cargo_bin = cargo_bin_dir()?;
    fs::create_dir_all(&cargo_bin)?;

    if cfg!(windows) {
        write_windows_shim(&cargo_bin, &workspace)?;
    } else {
        write_unix_shim(&cargo_bin, &workspace)?;
    }

    // Quiet on success: cargo:warning= shows as a rustc warning and looks like failure.
    Ok(())
}

fn cargo_bin_dir() -> io::Result<PathBuf> {
    if let Ok(home) = env::var("CARGO_HOME") {
        return Ok(PathBuf::from(home).join("bin"));
    }
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(to_io)?;
    Ok(PathBuf::from(home).join(".cargo").join("bin"))
}

fn write_windows_shim(cargo_bin: &Path, workspace: &Path) -> io::Result<()> {
    // Prefer .cmd over a stale cargo-install .exe (PATHEXT ranks .EXE before .CMD).
    let stale_exe = cargo_bin.join("ethean.exe");
    if stale_exe.is_file() {
        let _ = fs::remove_file(&stale_exe);
    }

    let root = workspace_display(workspace);
    let body = format!(
        "@echo off\r\n\
         setlocal\r\n\
         set \"ETHEAN_ROOT={root}\"\r\n\
         if exist \"%ETHEAN_ROOT%\\target\\release\\ethean.exe\" (\r\n\
           \"%ETHEAN_ROOT%\\target\\release\\ethean.exe\" %*\r\n\
           exit /b %ERRORLEVEL%\r\n\
         )\r\n\
         if exist \"%ETHEAN_ROOT%\\target\\debug\\ethean.exe\" (\r\n\
           \"%ETHEAN_ROOT%\\target\\debug\\ethean.exe\" %*\r\n\
           exit /b %ERRORLEVEL%\r\n\
         )\r\n\
         echo ethean: binary missing. Run: cargo build -p ethean --release\r\n\
         exit /b 127\r\n"
    );
    fs::write(cargo_bin.join("ethean.cmd"), body)?;
    Ok(())
}

fn write_unix_shim(cargo_bin: &Path, workspace: &Path) -> io::Result<()> {
    let root = workspace_display(workspace);
    let body = format!(
        "#!/bin/sh\n\
         set -eu\n\
         ETHEAN_ROOT=\"{root}\"\n\
         if [ -x \"$ETHEAN_ROOT/target/release/ethean\" ]; then\n\
           exec \"$ETHEAN_ROOT/target/release/ethean\" \"$@\"\n\
         fi\n\
         if [ -x \"$ETHEAN_ROOT/target/debug/ethean\" ]; then\n\
           exec \"$ETHEAN_ROOT/target/debug/ethean\" \"$@\"\n\
         fi\n\
         echo \"ethean: binary missing. Run: cargo build -p ethean --release\" >&2\n\
         exit 127\n"
    );
    let shim = cargo_bin.join("ethean");
    fs::write(&shim, body)?;
    set_executable(&shim)?;
    Ok(())
}

fn workspace_display(workspace: &Path) -> String {
    let s = workspace.display().to_string();
    // Escape for double-quoted shell / keep cmd paths usable.
    s.replace('\\', "/")
}

fn set_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn to_io(err: env::VarError) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, err)
}
