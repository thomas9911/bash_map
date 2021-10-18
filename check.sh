export wow='{"number":1, "text": "something"}'
# echo $wow
./target/debug/bash_map.exe get wow '\/number'
./target/debug/bash_map.exe get "$wow" '\/text'
