# Simple JSON (simjson)

## Purpose
Simple JSON is a parser in Rust and for Rust

## Usage
Just pass a JSON string to the crate as bellow,
```rust
extern crate simjson;
...

let json = simjson::parse(r#"[{"name":"Malvina\ud83d\udc67!", "age":19},{}, 45.8]"#);
println!{"{json:?}"}
```
Use something as below to extract a particular data,
```rust
extern crate simjson;
...

let json = simjson::parse(r#"{"name":"malvina", "parent":{"name": "Maria"}}"#);
println!("parent:{}", simjson::get_path_as_text(&json, &"parent/name").unwrap_or("unknown".to_string()));
```

## Microlibrary
This crate uses a consept of microlibray described in the [article](https://www.linkedin.com/pulse/micro-libraries-vs-mega-dmitriy-rogatkin-q6e6c).