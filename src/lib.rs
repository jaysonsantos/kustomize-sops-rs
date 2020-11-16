use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{self, Command},
};

use base64::encode;
use clap::Clap;
use color_eyre::{
    eyre::{eyre, Context},
    Help, Result, SectionExt,
};
use process::Stdio;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_yaml::Value;

pub const API_VERSION: &str = "kustomize-sops-rs/v1";
pub const SECRET_GENERATOR: &str = "SecretGenerator";
pub const SECRET_OUTPUT: &str = "Secret";
pub const CONFIG_MAP_OUTPUT: &str = "ConfigMap";
pub const ANNOTATIONS_KEY: &str = "annotations";
pub const ANNOTATION_NEEDS_HASH: &str = "kustomize.config.k8s.io/needs-hash";
pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

#[derive(Debug, Clap)]
pub enum SubCommand {
    Install,
}
#[derive(Debug, Clap)]
pub struct Arguments {
    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>,
    pub yaml_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Kind {
    SecretGenerator,
    ConfigMapGenerator,
    SimpleDecrypt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    String(String),
    HashMap(HashMap<String, String>),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Base {
    pub api_version: String,
    pub kind: Kind,
    pub metadata: HashMap<String, Metadata>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[serde(flatten)]
    pub base: Base,
    pub files: Vec<PathBuf>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseMapOutput {
    #[serde(flatten)]
    pub base: Base,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretOrConfigMap {
    #[serde(flatten)]
    pub base: Base,
    pub kind: String,
    pub data: HashMap<String, Value>,
}

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

pub fn generate_data_field(files: &Vec<PathBuf>) -> Result<HashMap<String, Value>> {
    let mut output_map = HashMap::new();

    for file in files {
        let decrypted: HashMap<String, Value> = decrypt_parse_yaml_file(&file)?;
        for (key, value) in decrypted.iter() {
            let value = match value {
                Value::Null => None,
                Value::Bool(v) => Some(format!("{}", v)),
                Value::Number(v) => Some(format!("{}", v)),
                Value::String(v) => Some(v.clone()),
                Value::Sequence(_) => todo!("Implement sequence"),
                Value::Mapping(_) => todo!("Implement mapping"),
            }
            .map(encode);
            let final_value = match value {
                Some(v) => Value::String(v),
                None => Value::Null,
            };
            output_map.insert(key.clone(), final_value.clone());
        }
    }
    Ok(output_map)
}

pub fn generate_output_map(
    kind: &str,
    data: HashMap<String, Value>,
    input: &Input,
) -> SecretOrConfigMap {
    let mut metadata = input.base.metadata.clone();
    let annotations = metadata
        .entry(ANNOTATIONS_KEY.into())
        .or_insert_with(|| Metadata::HashMap(HashMap::new()));

    match annotations {
        Metadata::String(_) => panic!("{} shoult not be a string at this point", ANNOTATIONS_KEY),
        Metadata::HashMap(h) => {
            h.insert(ANNOTATION_NEEDS_HASH.into(), "true".into());
        }
    }

    SecretOrConfigMap {
        base: Base {
            api_version: "v1".into(),
            metadata,
            kind: Kind::SecretGenerator,
            ..input.base
        },
        kind: kind.into(),
        data,
    }
}
