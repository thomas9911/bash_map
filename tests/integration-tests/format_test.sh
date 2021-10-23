#! /bin/bash

cargo build

alias map='./target/debug/bash_map.exe'

input="{\"boolean_value\":true,\"extra\":{\"test\":2100.0},\"nested\":{\"testing\":\"five\"},\"number\":1,\"text\":\"something\"}"

result=$(map --pretty get $input "")

expected="{
  \"boolean_value\": true,
  \"extra\": {
    \"test\": 2100.0
  },
  \"nested\": {
    \"testing\": \"five\"
  },
  \"number\": 1,
  \"text\": \"something\"
}"

if [ "$result" != "$expected" ]; then
    exit 1
fi
