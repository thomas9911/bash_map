#! /bin/bash

cargo build

alias map='./target/debug/bash_map.exe'

if [[ $(map init) != "{}" ]]; then
    exit 1
fi
