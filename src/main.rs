use serde_json::{from_str, to_string, Value};
use std::env::var;

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    command: MySubCommandEnum,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum MySubCommandEnum {
    Init(SubCommandInit),
    Get(SubCommandGet),
    Set(SubCommandSet),
    Type(SubCommandType),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Create empty map
#[argh(subcommand, name = "init")]
struct SubCommandInit {}

#[derive(FromArgs, PartialEq, Debug)]
/// Check the current variable on json type
#[argh(subcommand, name = "type")]
struct SubCommandType {
    #[argh(positional)]
    variable: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "get",
    description = "Get item from the map with given json pointer",
    note = "Check https://tools.ietf.org/html/rfc6901 for the spec on json pointer",
    example = r#"input                        pointer           output
{{"test": "input"}}            "/test"           "input"
{{"test": [1, 2, 3, 4]}}       "/test/2"         3
{{"test": [{{"sub": ["ok"]}}]}}  "/test/0/sub/0"   "ok""#
)]
struct SubCommandGet {
    #[argh(positional)]
    variable: String,
    #[argh(positional)]
    pointer: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "set",
    description = "Set the value or the object in variable at the given pointer",
    note = "Check https://tools.ietf.org/html/rfc6901 for the spec on json pointer",
    example = r#"input                    pointer          value      output
{{"test": "input"}}        "/test"          "input"    {{"test":"input"}}
{{}}                       "/test"          "input"    {{"test":"input"}}
{{}}                       "/test/key"      1.0        {{"test":{{"key":1.0}}}}"#
)]
struct SubCommandSet {
    #[argh(positional)]
    variable: String,
    #[argh(positional)]
    pointer: String,
    #[argh(positional, from_str_fn(value_from_str))]
    value: Value,
}

fn value_from_str(input: &str) -> Result<Value, String> {
    from_str(input).map_err(|x| x.to_string())
}

fn main() -> Result<(), String> {
    use MySubCommandEnum::*;
    let arg: TopLevel = argh::from_env();

    match arg.command {
        Init(_) => Ok(println!("{{}}")),
        Get(args) => Ok(println!("{}", do_get(args))),
        Set(args) => Ok(println!("{}", do_set(args))),
        Type(args) => Ok(println!("{}", do_type(args))),
    }
}

fn do_type(args: SubCommandType) -> String {
    use Value::*;

    match variable_or_value(&args.variable) {
        Null => "null",
        Bool(_) => "boolean",
        Number(_) => "number",
        String(_) => "string",
        Array(_) => "array",
        Object(_) => "object",
    }
    .to_string()
}

fn do_set(args: SubCommandSet) -> String {
    let mut value = variable_or_object(&args.variable);
    match pointer_mut(&mut value, &args.pointer.replace(r"\/", "/")) {
        Some(val) => {
            *val = args.value;
            to_string(&value).unwrap_or(String::new())
        }
        None => to_string(&value).unwrap_or(String::new()),
    }
}

pub fn pointer_mut<'a>(value: &'a mut Value, pointer: &str) -> Option<&'a mut Value> {
    // partial copy from https://github.com/serde-rs/json/blob/master/src/value/mod.rs
    if pointer.is_empty() {
        return Some(value);
    }
    if !pointer.starts_with('/') {
        return None;
    }
    pointer
        .split('/')
        .skip(1)
        .map(|x| x.replace("~1", "/").replace("~0", "~"))
        .try_fold(value, pointer_inner)
}

fn pointer_inner(target: &mut serde_json::Value, token: String) -> Option<&mut Value> {
    match target {
        Value::Object(map) => {
            map.entry(&token).or_insert(Value::Null);
            map.get_mut(&token)
        }
        Value::Array(list) => parse_index(&token).and_then(move |x| list.get_mut(x)),
        other => {
            let mut map = serde_json::Map::new();
            map.entry(&token).or_insert(Value::Null);
            *other = Value::Object(map);
            match other {
                Value::Object(map) => {
                    map.entry(&token).or_insert(Value::Null);
                    map.get_mut(&token)
                }
                _ => None,
            }
        }
    }
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
}

fn do_get(args: SubCommandGet) -> String {
    match variable_or_object(&args.variable).pointer(&args.pointer.replace(r"\/", "/")) {
        Some(val) => to_string(val).unwrap_or(String::new()),
        None => String::new(),
    }
}

fn variable_or_object(input: &str) -> Value {
    match from_str(input) {
        Ok(x) => Value::Object(x),
        _ => {
            let item = var(input).unwrap_or(String::new());
            from_str(&item).unwrap_or(Value::Object(Default::default()))
        }
    }
}

fn variable_or_value(input: &str) -> Value {
    match from_str(input) {
        Ok(x) => x,
        _ => {
            let item = var(input).unwrap_or(String::new());
            from_str(&item).unwrap_or(Value::Null)
        }
    }
}

#[cfg(test)]
mod doc_test {
    use super::{do_get, do_set, SubCommandGet, SubCommandSet};

    #[derive(Debug)]
    struct SetLine<'a> {
        input: &'a str,
        pointer: &'a str,
        value: &'a str,
        output: &'a str,
    }

    #[derive(Debug)]
    struct GetLine<'a> {
        input: &'a str,
        pointer: &'a str,
        output: &'a str,
    }

    fn parse_lines(input: &str) -> Vec<Vec<&str>> {
        let iter = input
            .lines()
            .skip_while(|x| !x.starts_with("Examples"))
            .skip(2)
            .take_while(|x| !x.starts_with("Notes"))
            .filter(|x| x != &"");

        let mut lines = Vec::new();
        for line in iter {
            let columns: Vec<_> = line
                .split("  ")
                .map(|x| x.trim())
                .filter(|x| x != &"")
                .collect();

            lines.push(columns)
        }
        lines
    }

    #[test]
    fn get_command() {
        use argh::FromArgs;

        let output = SubCommandGet::from_args(&[], &["--help"]).unwrap_err();

        let mut amount_of_lines = 0;

        for line in parse_lines(&output.output) {
            let line = GetLine {
                input: line[0],
                pointer: line[1],
                output: line[2],
            };

            let args =
                SubCommandGet::from_args(&[], &[line.input, &line.pointer.replace("\"", "")])
                    .unwrap();
            let output = do_get(args);

            assert_eq!(output, line.output);
            amount_of_lines += 1;
        }
        assert_eq!(amount_of_lines, 3)
    }

    #[test]
    fn set_command() {
        use argh::FromArgs;

        let output = SubCommandSet::from_args(&[], &["--help"]).unwrap_err();

        let mut amount_of_lines = 0;

        for line in parse_lines(&output.output) {
            let line = SetLine {
                input: line[0],
                pointer: line[1],
                value: line[2],
                output: line[3],
            };

            let args = SubCommandSet::from_args(
                &[],
                &[line.input, &line.pointer.replace("\"", ""), &line.value],
            )
            .unwrap();
            let output = do_set(args);

            assert_eq!(output, line.output);
            amount_of_lines += 1;
        }
        assert_eq!(amount_of_lines, 3)
    }
}

#[cfg(test)]
mod set_test {
    use super::{do_set, SubCommandSet};

    #[test]
    fn invalid_key_returns_the_input() {
        let data = serde_json::json!({
            "key": "number"
        })
        .to_string();

        assert_eq!(
            data.clone(),
            do_set(SubCommandSet {
                variable: data,
                pointer: "invalid key".to_string(),
                value: serde_json::json!(1.0)
            })
        );
    }

    #[test]
    fn replace_works() {
        let data = serde_json::json!({
            "key": "number"
        })
        .to_string();

        assert_eq!(
            serde_json::json!({
                "key": 1.0
            })
            .to_string(),
            do_set(SubCommandSet {
                variable: data,
                pointer: "/key".to_string(),
                value: serde_json::json!(1.0)
            })
        );
    }

    #[test]
    fn addition_works() {
        let data = serde_json::json!({
            "key": "number"
        })
        .to_string();

        assert_eq!(
            serde_json::json!({
                "key": "number",
                "other": 1.0
            })
            .to_string(),
            do_set(SubCommandSet {
                variable: data,
                pointer: "/other".to_string(),
                value: serde_json::json!(1.0)
            })
        );
    }

    #[test]
    fn nested_addition_works() {
        let data = serde_json::json!({
            "key": "number"
        })
        .to_string();

        assert_eq!(
            serde_json::json!({
                "key": "number",
                "nested": {"other": 1.0}
            })
            .to_string(),
            do_set(SubCommandSet {
                variable: data,
                pointer: "/nested/other".to_string(),
                value: serde_json::json!(1.0)
            })
        );
    }

    #[test]
    fn multi_nested_addition_works() {
        let data = serde_json::json!({
            "key": "number"
        })
        .to_string();

        assert_eq!(
            serde_json::json!({
                "key": "number",
                "a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": 1.0}}}}}}}
            })
            .to_string(),
            do_set(SubCommandSet {
                variable: data,
                pointer: "/a/b/c/d/e/f/g/h".to_string(),
                value: serde_json::json!(1.0)
            })
        );
    }
}

#[cfg(test)]
mod get_test {
    use super::{do_get, SubCommandGet};

    #[test]
    fn escaped_key() {
        let data = serde_json::json!({
            "key": "number"
        })
        .to_string();

        assert_eq!(
            r#""number""#,
            do_get(SubCommandGet {
                variable: data,
                pointer: "\\/key".to_string()
            })
        );
    }

    #[test]
    fn works() {
        let data = serde_json::json!({
            "key": "number"
        })
        .to_string();

        assert_eq!(
            r#""number""#,
            do_get(SubCommandGet {
                variable: data,
                pointer: "/key".to_string()
            })
        );
    }

    #[test]
    fn slice_array() {
        let data = serde_json::json!({
            "key": [
                "one",
                "two",
                "three"
            ]
        })
        .to_string();

        assert_eq!(
            r#""two""#,
            do_get(SubCommandGet {
                variable: data.to_string(),
                pointer: "/key/1".to_string()
            })
        );

        assert_eq!(
            r#""three""#,
            do_get(SubCommandGet {
                variable: data.to_string(),
                pointer: "/key/2".to_string()
            })
        );
    }

    #[test]
    fn nested() {
        let data = serde_json::json!({
            "key": [
                {"one": 1},
                {"two": 2},
                {"three": 3}
            ]
        })
        .to_string();

        assert_eq!(
            r#"3"#,
            do_get(SubCommandGet {
                variable: data.to_string(),
                pointer: "/key/2/three".to_string()
            })
        );

        assert_eq!(
            r#"2"#,
            do_get(SubCommandGet {
                variable: data.to_string(),
                pointer: "/key/1/two".to_string()
            })
        );

        assert_eq!(
            r#"{"one":1}"#,
            do_get(SubCommandGet {
                variable: data.to_string(),
                pointer: "/key/0".to_string()
            })
        );
    }
}

#[cfg(test)]
mod type_test {
    use super::{do_type, SubCommandType};

    #[test]
    fn number() {
        assert_eq!(
            "number",
            do_type(SubCommandType {
                variable: "1.123".to_string()
            })
        );
        assert_eq!(
            "number",
            do_type(SubCommandType {
                variable: "1".to_string()
            })
        );
        assert_eq!(
            "number",
            do_type(SubCommandType {
                variable: "3e-12".to_string()
            })
        );
        assert_eq!(
            "number",
            do_type(SubCommandType {
                variable: "-2.1e5".to_string()
            })
        );
    }

    #[test]
    fn object() {
        assert_eq!(
            "object",
            do_type(SubCommandType {
                variable: "{}".to_string()
            })
        );
        assert_eq!(
            "object",
            do_type(SubCommandType {
                variable: "{\"key\": 123}".to_string()
            })
        );
    }

    #[test]
    fn array() {
        assert_eq!(
            "array",
            do_type(SubCommandType {
                variable: "[]".to_string()
            })
        );
        assert_eq!(
            "array",
            do_type(SubCommandType {
                variable: "[1,2,3,4]".to_string()
            })
        );
    }

    #[test]
    fn null() {
        assert_eq!(
            "null",
            do_type(SubCommandType {
                variable: "null".to_string()
            })
        );
        assert_eq!(
            "null",
            do_type(SubCommandType {
                variable: "".to_string()
            })
        );
        assert_eq!(
            "null",
            do_type(SubCommandType {
                variable: "unknown_variable".to_string()
            })
        );
        assert_eq!(
            "null",
            do_type(SubCommandType {
                variable: "{not json ".to_string()
            })
        );
    }

    #[test]
    fn boolean() {
        assert_eq!(
            "boolean",
            do_type(SubCommandType {
                variable: "true".to_string()
            })
        );
        assert_eq!(
            "boolean",
            do_type(SubCommandType {
                variable: "false".to_string()
            })
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            "string",
            do_type(SubCommandType {
                variable: r#""test""#.to_string()
            })
        );
        assert_eq!(
            "string",
            do_type(SubCommandType {
                variable: r#""false""#.to_string()
            })
        );
        assert_eq!(
            "string",
            do_type(SubCommandType {
                variable: r#""1.123""#.to_string()
            })
        );
        assert_eq!(
            "string",
            do_type(SubCommandType {
                variable: "\"string\"".to_string()
            })
        );
    }

    #[test]
    fn from_env_var() {
        use std::env::set_var;

        set_var("testing_var", r#"{"key": "value"}"#);
        assert_eq!(
            "object",
            do_type(SubCommandType {
                variable: "testing_var".to_string()
            })
        );
        set_var("testing_var", r#""string""#);
        assert_eq!(
            "string",
            do_type(SubCommandType {
                variable: "testing_var".to_string()
            })
        );
        set_var("testing_var", "1.123");
        assert_eq!(
            "number",
            do_type(SubCommandType {
                variable: "testing_var".to_string()
            })
        );
    }
}