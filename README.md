# kustomize-sops-rs
Kustomize (exec) plugin to generate secrets/config map from encrypted .env files and simple decrypter
## Requirements
It basically needs `sops` binary in your path to work and to run the tests, gpg is also required.
## Installing
Just run the following script and it should place the binary on `/usr/local/bin` and it creates the kustomize structure to host the plugin.
```bash
curl -sL https://github.com/jaysonsantos/kustomize-sops-rs/raw/main/install.sh | bash -s
```

The output should be like this:
```
./install.sh
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  8139    0  8139    0     0  25675      0 --:--:-- --:--:-- --:--:-- 26003
Downloading binary https://github.com/jaysonsantos/kustomize-sops-rs/releases/download/v0.1.0/kustomize-sops-x86_64-unknown-linux-musl.gz
Done
Install kustomize-sops-x86_64-unknown-linux-musl to /usr/local/bin/kustomize-sops
Linking plugins
Linking kustomize-sops-rs to /home/jayson/.config/kustomize/plugin/kustomize-sops-rs/v1/configmapgenerator/ConfigMapGenerator
Linking kustomize-sops-rs to /home/jayson/.config/kustomize/plugin/kustomize-sops-rs/v1/secretgenerator/SecretGenerator
Linking kustomize-sops-rs to /home/jayson/.config/kustomize/plugin/kustomize-sops-rs/v1/simpledecrypt/SimpleDecrypt
```

## Usage
This is a generator so your kustomize file should have something along these lines.
```yaml
generators:
  - secrets.yaml

```
and the secrets file
```yaml
apiVersion: kustomize-sops-rs/v1
kind: SecretGenerator
metadata:
  name: secrets
files:
  - encrypted.yaml
---
apiVersion: kustomize-sops-rs/v1
kind: ConfigMapGenerator
metadata:
  name: config-map
files:
  - encrypted.yaml
---
apiVersion: kustomize-sops-rs/v1
kind: SimpleDecrypt
metadata:
  name: simple-decrypt
files:
  - ingress.enc.yaml
```
The kinds `SecretGenerator` and `ConfigMapGenerator` should generate `Secret` and `ConfigMap` the same
way kustomize does (with the shiny hashes) but it reads an yaml file with **one level of mapping for now**.
To test it, create an encrypted file with sops using the following command (assuming you imported the private key from tests folder)
```bash
printf "key: value\npassword: protected\n" | \
sops -p EBC846D0169D43A96ABA1C31AD471BDF8E8A0484 \
     -e --input-type yaml --output-type yaml \
     /dev/stdin > encrypted.yaml
```

After this you could just run `kustomize build --enable_alpha_plugins folder` and it should generate your final yaml.
The kind `SimpleDecrypt` will just decrypt the file and pass it along, so it has to be a valid kubernetes object as you will probably apply it.
