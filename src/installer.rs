#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
#[cfg(target_family = "windows")]
use std::os::windows::fs::symlink_file as symlink;

use std::path::MAIN_SEPARATOR;
use std::{
    env::{args, var},
    fs::{canonicalize, create_dir_all},
    path::PathBuf,
};

use color_eyre::eyre::{eyre, WrapErr};
use dirs::home_dir;

use crate::types::Kind;
use crate::{API_VERSION, XDG_CONFIG_HOME};

pub fn install() -> color_eyre::Result<()> {
    let home = home_dir().ok_or_else(|| eyre!("Failed to determine home director"))?;
    let install_directory = get_install_directory(home);

    let source = PathBuf::from(args().next().unwrap());
    let source =
        canonicalize(&source).wrap_err("failed to find the absolute path of the current binary")?;

    let kinds = [
        Kind::ConfigMapGenerator,
        Kind::SecretGenerator,
        Kind::SimpleDecrypt,
    ];

    let binary_suffix = if cfg!(windows) { ".exe" } else { "" };

    for kind in &kinds {
        let kind = format!("{:?}", kind);
        let destination_folder = install_directory.join(&kind.to_lowercase());
        create_dir_all(&destination_folder).wrap_err_with(|| {
            format!(
                "failed to create directory {}",
                &destination_folder.to_string_lossy()
            )
        })?;

        let destination = destination_folder.join(format!("{}{}", &kind, &binary_suffix));
        println!(
            "Linking kustomize-sops-rs to {}",
            &destination.to_string_lossy()
        );
        if destination.exists() {
            std::fs::remove_file(&destination).wrap_err("failed to delete old file")?;
        }
        symlink(&source, destination).wrap_err("failed to create link")?;
    }
    Ok(())
}

fn get_install_directory(home: PathBuf) -> PathBuf {
    let api_directory = API_VERSION.replace("/", &MAIN_SEPARATOR.to_string());
    var(XDG_CONFIG_HOME)
        .wrap_err("failed to get the install directory")
        .map(|config| PathBuf::from(config))
        .unwrap_or_else(|_| home.join(".config"))
        .join("kustomize")
        .join("plugin")
        .join(api_directory)
}
