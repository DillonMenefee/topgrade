#[cfg(target_os = "linux")]
use crate::error::SkipStep;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use anyhow::Result;
use directories::BaseDirs;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_fisher(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let fish = require("fish")?;
    base_dirs
        .home_dir()
        .join(".config/fish/functions/fisher.fish")
        .require()?;

    print_separator("Fisher");
    run_type
        .execute(&fish)
        .args(&["-c", "fisher self-update"])
        .check_run()?;

    run_type.execute(&fish).args(&["-c", "fisher"]).check_run()
}

pub fn run_homebrew(cleanup: bool, run_type: RunType) -> Result<()> {
    let brew = require("brew")?;
    print_separator("Brew");

    run_type.execute(&brew).arg("update").check_run()?;
    run_type.execute(&brew).arg("upgrade").check_run()?;

    let cask_upgrade_exists = Command::new(&brew)
        .args(&["--repository", "buo/cask-upgrade"])
        .check_output()
        .map(|p| Path::new(p.trim()).exists())?;

    if cask_upgrade_exists {
        run_type.execute(&brew).args(&["cu", "-ay"]).check_run()?;
    } else {
        run_type.execute(&brew).args(&["cask", "upgrade"]).check_run()?;
    }

    if cleanup {
        run_type.execute(&brew).arg("cleanup").check_run()?;
    }

    Ok(())
}

pub fn run_nix(run_type: RunType) -> Result<()> {
    let nix = require("nix")?;
    let nix_channel = require("nix-channel")?;
    let nix_env = require("nix-env")?;
    print_separator("Nix");

    #[cfg(target_os = "linux")]
    {
        use super::linux::Distribution;
        use log::debug;

        if let Ok(Distribution::NixOS) = Distribution::detect() {
            debug!("Nix on NixOS must be upgraded via 'nixos-rebuild switch', skipping.");
            return Err(SkipStep.into());
        }
    }

    run_type.execute(&nix).arg("upgrade-nix").check_run()?;
    run_type.execute(&nix_channel).arg("--update").check_run()?;
    run_type.execute(&nix_env).arg("--upgrade").check_run()
}

pub fn run_home_manager(run_type: RunType) -> Result<()> {
    let home_manager = require("home-manager")?;

    print_separator("home-manager");
    run_type.execute(&home_manager).arg("switch").check_run()
}

pub fn run_pearl(run_type: RunType) -> Result<()> {
    let pearl = require("pearl")?;
    print_separator("pearl");

    run_type.execute(&pearl).arg("update").check_run()
}

pub fn run_sdkman(base_dirs: &BaseDirs, cleanup: bool, run_type: RunType) -> Result<()> {
    let bash = require("bash")?;

    let sdkman_init_path = env::var("SDKMAN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join(".sdkman"))
        .join("bin")
        .join("sdkman-init.sh")
        .require()
        .map(|p| format!("{}", &p.display()))?;

    print_separator("SDKMAN!");

    let cmd_selfupdate = format!("source {} && sdk selfupdate", &sdkman_init_path);
    run_type
        .execute(&bash)
        .args(&["-c", cmd_selfupdate.as_str()])
        .check_run()?;

    let cmd_update = format!("source {} && sdk update", &sdkman_init_path);
    run_type.execute(&bash).args(&["-c", cmd_update.as_str()]).check_run()?;

    let cmd_upgrade = format!("source {} && sdk upgrade", &sdkman_init_path);
    run_type
        .execute(&bash)
        .args(&["-c", cmd_upgrade.as_str()])
        .check_run()?;

    if cleanup {
        let cmd_flush_archives = format!("source {} && sdk flush archives", &sdkman_init_path);
        run_type
            .execute(&bash)
            .args(&["-c", cmd_flush_archives.as_str()])
            .check_run()?;

        let cmd_flush_temp = format!("source {} && sdk flush temp", &sdkman_init_path);
        run_type
            .execute(&bash)
            .args(&["-c", cmd_flush_temp.as_str()])
            .check_run()?;
    }

    Ok(())
}

pub fn reboot() {
    print!("Rebooting...");
    Command::new("sudo").arg("reboot").spawn().unwrap().wait().unwrap();
}
