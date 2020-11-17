use std::{
    path::Path,
    process::{Command, Stdio},
};

use color_eyre::{
    eyre::{eyre, Context},
    Help, Result, SectionExt,
};
use serde::de::DeserializeOwned;

pub fn decrypt_file(file: &Path) -> Result<Vec<u8>> {
    let output = Command::new("sops")
        .args(&["--decrypt", &*file.to_string_lossy(), "/dev/stdout"])
        .stdout(Stdio::piped())
        .output()
        .wrap_err("failed to run sops")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("gpg exited with non-zero status code"))
            .with_section(move || stdout.trim().to_string().header("Stdout:"))
            .with_section(move || stderr.trim().to_string().header("Stderr:"));
    }
    Ok(output.stdout)
}

pub fn decrypt_parse_yaml_file<T>(file: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_yaml::from_slice(&decrypt_file(file)?)?)
}
