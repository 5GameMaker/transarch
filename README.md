# Transarch
<center>Cargo interstate</center>

Cross-compile Rust code into a blob

```rust
let wasm_blob = wasm! { // Happy wasm blob
    pub extern "C" fn add(a: i32, b: i32) {
        a + b
    }
};
```

This crate uses all the bad practices to compile Rust code into a blob. Be sure to `cargo clean` once in a while!
