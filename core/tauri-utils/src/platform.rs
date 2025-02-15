// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Platform helper functions.

use std::{
  env,
  path::{PathBuf, MAIN_SEPARATOR},
};

use crate::PackageInfo;

/// Try to determine the current target triple.
///
/// Returns a target triple (e.g. `x86_64-unknown-linux-gnu` or `i686-pc-windows-msvc`) or an
/// `Error::Config` if the current config cannot be determined or is not some combination of the
/// following values:
/// `linux, mac, windows` -- `i686, x86, armv7` -- `gnu, musl, msvc`
///
/// * Errors:
///     * Unexpected system config
pub fn target_triple() -> crate::Result<String> {
  let arch = if cfg!(target_arch = "x86") {
    "i686"
  } else if cfg!(target_arch = "x86_64") {
    "x86_64"
  } else if cfg!(target_arch = "arm") {
    "armv7"
  } else if cfg!(target_arch = "aarch64") {
    "aarch64"
  } else {
    return Err(crate::Error::Architecture);
  };

  let os = if cfg!(target_os = "linux") {
    "unknown-linux"
  } else if cfg!(target_os = "macos") {
    "apple-darwin"
  } else if cfg!(target_os = "windows") {
    "pc-windows"
  } else if cfg!(target_os = "freebsd") {
    "unknown-freebsd"
  } else {
    return Err(crate::Error::Os);
  };

  let os = if cfg!(target_os = "macos") || cfg!(target_os = "freebsd") {
    String::from(os)
  } else {
    let env = if cfg!(target_env = "gnu") {
      "gnu"
    } else if cfg!(target_env = "musl") {
      "musl"
    } else if cfg!(target_env = "msvc") {
      "msvc"
    } else {
      return Err(crate::Error::Environment);
    };

    format!("{}-{}", os, env)
  };

  Ok(format!("{}-{}", arch, os))
}

/// Computes the resource directory of the current environment.
///
/// On Windows, it's the path to the executable.
///
/// On Linux, when running in an AppImage the `APPDIR` variable will be set to
/// the mounted location of the app, and the resource dir will be
/// `${APPDIR}/usr/lib/${exe_name}`. If not running in an AppImage, the path is
/// `/usr/lib/${exe_name}`.  When running the app from
/// `src-tauri/target/(debug|release)/`, the path is
/// `${exe_dir}/../lib/${exe_name}`.
///
/// On MacOS, it's `${exe_dir}../Resources` (inside .app).
pub fn resource_dir(package_info: &PackageInfo) -> crate::Result<PathBuf> {
  let exe = std::env::current_exe()?;
  let exe_dir = exe.parent().expect("failed to get exe directory");
  let curr_dir = exe_dir.display().to_string();

  if curr_dir.ends_with(format!("{S}target{S}debug", S = MAIN_SEPARATOR).as_str())
    || curr_dir.ends_with(format!("{S}target{S}release", S = MAIN_SEPARATOR).as_str())
    || cfg!(target_os = "windows")
  {
    // running from the out dir or windows
    return Ok(exe_dir.to_path_buf());
  }

  if cfg!(target_os = "linux") {
    if curr_dir.ends_with("/data/usr/bin") {
      // running from the deb bundle dir
      Ok(exe_dir.join(format!("../lib/{}", package_info.package_name())))
    } else if let Ok(appdir) = env::var("APPDIR") {
      Ok(PathBuf::from(format!(
        "{}/usr/lib/{}",
        appdir,
        package_info.package_name()
      )))
    } else {
      // running bundle
      Ok(PathBuf::from(format!(
        "/usr/lib/{}",
        package_info.package_name()
      )))
    }
  } else if cfg!(target_os = "macos") {
    Ok(exe_dir.join("../Resources"))
  } else {
    Err(crate::Error::UnsupportedPlatform)
  }
}
