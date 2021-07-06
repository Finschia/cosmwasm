#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

packages=("cosmwasm-crypto" "cosmwasm-derive" "cosmwasm-schema" "cosmwasm-std" "cosmwasm-storage" "cosmwasm-vm")

V=""

for ((i = 0; i < ${#packages[@]}; i++)) {
    V=$(cargo tree -i "${packages[i]}" | grep -o -E "([0-9]+\.){1}[0-9]+(\.[0-9]+)-([0-9]+\.){1}[0-9]+(\.[0-9]+)?" | head -n1)
    if [ "$1" != "$V" ]; then
        echo "mismatch version"
        exit 1
    fi
}

git tag "$V"
