#! /bin/bash

cargo build

alias map='./target/debug/bash_map.exe'

export result='{"number":1, "text": "something", "boolean_value": true}'

one=$(map get result '\/number')
bool=$(map get result '\/boolean_value')
typing=$(map type $bool)

if [ $typing == "boolean" ] && [ $one -eq 1 ] && [ $(map get "$result" '\/text') == '"something"' ]; then
    result=$(map set result '\/extra' '{"test": 2.1e3}')
    result=$(map set result '\/nested/testing' '"five"')

    expected="{\"boolean_value\":true,\"extra\":{\"test\":2100.0},\"nested\":{\"testing\":\"five\"},\"number\":1,\"text\":\"something\"}"

    if [ "$result" != "$expected" ]; then
        exit 1
    fi
else
    exit 1
fi
