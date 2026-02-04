# Simple JSON (simjson)

## Purpose
Simple JSON is a parser in Rust and for Rust

## Usage
Just pass a JSON string to the crate as bellow,
```rust
extern crate simjson;
...

let json = simjson::parse(r#"[{"name":"MalvikaII\ud83d\udc67!", "age":19},{}, 45.8]"#);
println!{"{json:?}"}
```
Use something as below to extract a particular data,
```rust
extern crate simjson;
...

let json = simjson::parse(r#"{"name":"Malvika", "parent":{"name": "Maria"}}"#);
println!("parent:{}", simjson::get_path_as_text(&json, &"parent/name").unwrap_or_else(|| "undefined".to_string()));
```

## Build
Use [RustBee](https://github.com/vernisaz/rust_bee) to build the crate. Script [bee.7b](./bee.7b) is provided.
Modify `crate_dir` if you use its other location than the specified.

Cargo manifest *toml* can be added in a case of Cargo usage, for example,
```cargo
[package]
name = "simjson"
version = "1.02:016"
edition = "2024"

[env]
VERSION = "1.02:016"

[dependencies]
```

## Microlibrary
This crate uses the concept of the Microlibrary described in the [article](https://www.linkedin.com/pulse/micro-libraries-vs-mega-dmitriy-rogatkin-q6e6c).