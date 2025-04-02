# Simple JSON (simjson)

## Purpose
Simple JSON parser for Rust

## Usage
Just pass a JSON string to the crate as bellow
```
extern crate simjson;
...

let json = simjson::parse("[{\"name\":\"malina\", \"age\":19},{}, 45.8]");
println!{"{json:?}"}
```

## Microlibrary
This crate uses a consept of microlibray described in the [article](https://www.linkedin.com/pulse/micro-libraries-vs-mega-dmitriy-rogatkin-q6e6c).