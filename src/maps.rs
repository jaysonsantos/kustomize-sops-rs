use std::{collections::HashMap, path::PathBuf};

use base64::encode;
use color_eyre::Result;
use serde_yaml::Value;

use crate::{
    decryption::decrypt_parse_yaml_file,
    types::Base,
    types::Metadata,
    types::{Input, Kind, SecretOrConfigMap},
    ANNOTATIONS_KEY, ANNOTATION_NEEDS_HASH,
};

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
