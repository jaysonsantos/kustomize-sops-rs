pub mod cli;
pub mod decryption;
pub mod maps;
pub mod types;

pub const API_VERSION: &str = "kustomize-sops-rs/v1";
pub const SECRET_OUTPUT: &str = "Secret";
pub const CONFIG_MAP_OUTPUT: &str = "ConfigMap";
pub const ANNOTATIONS_KEY: &str = "annotations";
pub const ANNOTATION_NEEDS_HASH: &str = "kustomize.config.k8s.io/needs-hash";
pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
