use std::{env::current_dir, process::Command, process::Output, process::Stdio, sync::Once};

use color_eyre::eyre::{eyre, Context};
use color_eyre::{Help, Result, SectionExt};

use kustomize_sops::XDG_CONFIG_HOME;

static SETUP_TESTS: Once = Once::new();

fn setup_tests() {
    SETUP_TESTS.call_once(|| {
        color_eyre::install().unwrap();
        configure_gpg().unwrap();
        install().unwrap();
    })
}

fn configure_gpg() -> Result<()> {
    let output = Command::new("gpg")
        .args(&["--import", "tests/kustomization/private.key"])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .wrap_err("failed to run gpg")?;
    check_output_status(&output)?;
    Ok(())
}

fn install() -> Result<()> {
    let debug_directory = current_dir()?.join("target").join("debug");
    let output = Command::new(debug_directory.join("kustomize-sops"))
        .env(XDG_CONFIG_HOME, debug_directory.as_os_str())
        .arg("install")
        .output()?;

    check_output_status(&output)?;
    Ok(())
}

fn check_output_status(output: &Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(eyre!("command exited with non-zero status code"))
        .with_section(move || stdout.trim().to_string().header("Stdout:"))
        .with_section(move || stderr.trim().to_string().header("Stderr:"));
}

#[test]
fn run_with_kustomize() {
    setup_tests();
    let debug_directory = current_dir().unwrap().join("target").join("debug");
    let output = Command::new("kustomize")
        .env(XDG_CONFIG_HOME, &*debug_directory.to_string_lossy())
        .args(&["build", "--enable_alpha_plugins", "tests/kustomization"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!(
            "kustomize exited with non-zero status code\nStdout: {}\nStderr: {}\n",
            stdout, stderr
        );
    }
    let expected = r#"apiVersion: v1
data:
  port: ODA4MA==
  test: dXNlcg==
kind: ConfigMap
metadata:
  name: config-map-9c8d7tcc49
---
apiVersion: v1
data:
  port: ODA4MA==
  test: dXNlcg==
kind: Secret
metadata:
  name: secrets-t4t69m8dbm
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-production
    ingress.kubernetes.io/force-ssl-redirect: "true"
    kubernetes.io/ingress.class: contour
    kubernetes.io/tls-acme: "true"
  name: ingress-test
spec:
  rules:
  - host: ingress.test.com
    http:
      paths:
      - backend:
          service:
            name: ingress-test
            port:
              number: 80
        path: /
        pathType: Prefix
  tls:
  - hosts:
    - ingress.test.com
    secretName: ingress-tls
"#;
    assert_eq!(&expected, &stdout);
}
