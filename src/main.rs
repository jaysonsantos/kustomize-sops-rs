use std::{
    env::{args, var},
    fs::{canonicalize, create_dir_all, File},
    io::{stdout, Write},
    os::unix::fs::symlink,
    path::PathBuf,
    process::exit,
};

use clap::Parser;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use dirs::home_dir;
use kustomize_sops::{
    cli::Arguments, cli::SubCommand, decryption::decrypt_file, maps::generate_data_field,
    maps::generate_output_map, types::Input, types::Kind, API_VERSION, CONFIG_MAP_OUTPUT,
    SECRET_OUTPUT, XDG_CONFIG_HOME,
};
use serde_yaml::{from_reader, to_string};

fn main() -> Result<()> {
    color_eyre::install()?;
    let arguments: Arguments = Arguments::parse();
    if let Some(yaml_file) = arguments.yaml_file {
        let input: Input = from_reader(File::open(yaml_file).unwrap()).unwrap();

        return match input.base.kind {
            Kind::SecretGenerator => process_map(SECRET_OUTPUT, &input),
            Kind::ConfigMapGenerator => process_map(CONFIG_MAP_OUTPUT, &input),
            Kind::SimpleDecrypt => process_simple_decrypt(&input),
        };
    }
    match arguments.subcommand {
        Some(SubCommand::Install) => return install(),
        None => {
            eprintln!("The yaml file is required if no command is set");
            exit(1);
        }
    }
}

fn process_map(kind: &str, input: &Input) -> Result<()> {
    let data = generate_data_field(&input.files)?;
    let output = generate_output_map(kind, data, &input);

    println!("{}", to_string(&output)?);
    Ok(())
}

fn process_simple_decrypt(input: &Input) -> Result<()> {
    let output = stdout();
    let mut output = output.lock();

    for file in &input.files {
        let decrypted = decrypt_file(&file)?;
        output.write_all(&decrypted)?;
        output.write_all(b"\n---\n")?;
    }

    Ok(())
}

fn install() -> Result<()> {
    let home = home_dir().ok_or_else(|| eyre!("Failed to determine home director"))?;
    let install_directory = var(XDG_CONFIG_HOME)
        .wrap_err("failed to get the install directory")
        .map(|config| PathBuf::from(config))
        .unwrap_or_else(|_| home.join(".config"))
        .join("kustomize")
        .join("plugin")
        .join(API_VERSION);

    let source = PathBuf::from(args().next().unwrap());
    let source =
        canonicalize(&source).wrap_err("failed to find the absolute path of the current binary")?;

    let kinds = [
        Kind::ConfigMapGenerator,
        Kind::SecretGenerator,
        Kind::SimpleDecrypt,
    ];

    for kind in &kinds {
        let kind = format!("{:?}", kind);
        let destination_folder = install_directory.join(&kind.to_lowercase());
        create_dir_all(&destination_folder).wrap_err_with(|| {
            format!(
                "failed to create directory {}",
                &destination_folder.to_string_lossy()
            )
        })?;
        let destination = destination_folder.join(&kind);
        println!(
            "Linking kustomize-sops-rs to {}",
            &destination.to_string_lossy()
        );
        if !destination.exists() {
            // XXX: To implement on windows this needs to change
            symlink(&source, destination).wrap_err("failed to create link")?;
        }
    }
    Ok(())
}
