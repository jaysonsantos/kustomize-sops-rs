use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

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
