use std::{
    fs::File,
    io::{stdout, Write},
    process::exit,
};

use clap::Clap;
use color_eyre::Result;
use serde_yaml::{from_reader, to_string};

use kustomize_sops::{
    cli::Arguments, cli::SubCommand, decryption::decrypt_file, installer,
    maps::generate_data_field, maps::generate_output_map, types::Input, types::Kind,
    CONFIG_MAP_OUTPUT, SECRET_OUTPUT,
};

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
        Some(SubCommand::Install) => return installer::install(),
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
