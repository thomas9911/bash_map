cargo build

alias map='./target/debug/bash_map.exe'

export result='{"number":1, "text": "something", "boolean_value": true}'

one=$(map get result '\/number')
bool=$(map get result '\/boolean_value')
typing=$(map type $bool)

if [ $typing == "boolean" ] && [ $one -eq 1 ] && [ $(map get "$result" '\/text') == '"something"' ]; then
    result=$(map set result '\/extra' '{"test": 2.1e3}')
    result=$(map set result '\/nested/testing' '"five"')
    echo $result
fi
