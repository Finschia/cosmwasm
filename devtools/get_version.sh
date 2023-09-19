#!/bin/bash
packages=("cosmwasm-derive" "cosmwasm-schema" "cosmwasm-std" "cosmwasm-storage" "cosmwasm-vm")

V=$(cargo tree -i "cosmwasm-crypto" | grep -o -E "\d+(\.\d+){2}\+\d+(\.\d+){2}" | head -n1)

for ((i = 0; i < ${#packages[@]}; i++)) {
    if [ "$V" != $(cargo tree -i "${packages[i]}" | grep -o -E "\d+(\.\d+){2}\+\d+(\.\d+){2}" | head -n1) ]; then
        echo "mismatch version"
        exit 1
    fi
}

echo "$V"
