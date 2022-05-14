#!/usr/bin/env bash
set -e
set -o pipefail

PROJECT_RELEASE_URL=https://api.github.com/repos/jaysonsantos/kustomize-sops-rs/releases

function get_binary_name {
    local arch
    local os
    local family
    local libc

    arch=$(uname -m | sed 's/arm64/aarch64/')
    os=$(uname -s | tr '[:upper:]' '[:lower:]')

    if [ "$os" = "linux" ]; then
        family=unknown
        libc=-musl
    else
        family=apple
    fi

    echo "kustomize-sops-${arch}-${family}-${os}${libc}"
}

function download_binary {
    local name
    local download_url

    name="$1"
    download_url=$(curl "$PROJECT_RELEASE_URL" | grep -o "https://.*/${name}")

    echo "Downloading binary $download_url"
    curl -Ls "$download_url" -o "$name"
    echo "Done"
}

function install_kustomize_sops {
    local binary_name
    local compressed_file
    local destination_file

    binary_name=$(get_binary_name)
    compressed_file=${binary_name}.gz

    download_binary "$compressed_file"
    gunzip "$compressed_file"

    destination_file="/usr/local/bin/kustomize-sops"

    echo "Install ${binary_name} to ${destination_file}"
    sudo install -m 755 "$binary_name" "$destination_file"

    echo "Linking plugins"
    "$destination_file" install
    rm -rf "$compressed_file" "$binary_name"
}

install_kustomize_sops
