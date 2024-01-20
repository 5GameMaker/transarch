use std::{
    env::current_dir,
    fs::{copy, create_dir, create_dir_all, read_dir, write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::Mutex,
};

use proc_macro::TokenStream;

extern crate proc_macro;

const CARGO_TOML_CONTENTS: &str = r#"[package]
name = "transarch-tmp-pkg"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[workspace]
members = []
"#;

static mut COMP_BLOB_ID: Mutex<usize> = Mutex::new(0);

fn target_dir() -> PathBuf {
    current_dir()
        .unwrap()
        .ancestors()
        .find(|x| x.join("Cargo.toml").is_file() && x.join("target").is_dir())
        .map(|x| x.join("target"))
        .expect("no target dir found. what did you do?")
}

fn build(tokens: TokenStream, target: Option<String>) -> (PathBuf, String) {
    let dir = target_dir().join("transarch/transarch-tmp-crate");

    if !dir.exists() {
        create_dir_all(dir.clone()).expect("setup failed: failed to create <TRANSARCH-ROOT>");
        create_dir(dir.join("src")).expect("setup failed: failed to create <TRANSARCH-ROOT>/src");
        write(dir.join("Cargo.toml"), CARGO_TOML_CONTENTS)
            .expect("setup failed: failed to create <TRANSARCH-ROOT>/Cargo.toml");
    }

    let mut iter = tokens.into_iter();

    let target = target.unwrap_or_else(|| {
        iter.next()
            .and_then(|x| match x {
                proc_macro::TokenTree::Literal(x) => {
                    let x = x.to_string();
                    if !x.starts_with('"') || !x.is_ascii() {
                        panic!("target must be a string");
                    }

                    Some(x[1..x.len() - 1].to_string())
                }
                _ => None,
            })
            .expect("no target provided")
    });

    write(
        dir.join("src/lib.rs"),
        TokenStream::from_iter(iter).to_string(),
    )
    .expect("setup failed: failed to create <TRANSARCH-ROOT>/src/lib.rs");

    let mut cmd = Command::new("cargo")
        .arg("build")
        .arg(format!("--target={target}"))
        .arg("--color=always")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(&dir)
        .spawn()
        .expect("failed to run cargo");

    if !cmd.wait().expect("how").success() {
        panic!("failed to execute cargo");
    }

    (dir, target)
}

#[proc_macro]
/// Cross-compile into a target and obtain contents of build directory
/// ```rust
/// let dir = cross! {
///     "wasm32-unknown-unknown"
///     pub fn hi() -> String {
///         "hi".to_string()
///     }
/// };
/// ```
pub fn cross(tokens: TokenStream) -> TokenStream {
    let mut id = unsafe { COMP_BLOB_ID.lock().unwrap() };

    let (dir, target) = build(tokens, None);

    let out_dir = dir.join(format!("target/{target}/debug"));

    let mut buf = String::new();

    fn make(buf: &mut String, path: PathBuf, out_dir: PathBuf, blob_id: &mut usize) {
        if path.is_dir() {
            buf.push_str("Into::<::transarch::Dir>::into({");
            buf.push_str("let mut map=::std::collections::HashMap::new();");
            for x in read_dir(path).unwrap() {
                let x = x.unwrap();
                if x.path().is_dir() {
                    buf.push_str(&format!(
                        r#"map.insert("{}",::transarch::Entry::Dir("#,
                        x.file_name().into_string().unwrap()
                    ));
                } else {
                    buf.push_str(&format!(
                        r#"map.insert("{}",::transarch::Entry::File("#,
                        x.file_name().into_string().unwrap()
                    ));
                }
                make(buf, x.path(), out_dir.clone(), blob_id);
                buf.push_str("));");
            }
            buf.push_str("map})");
            return;
        }

        let file = out_dir.join(format!("../blob{}", *blob_id));
        copy(path, &file).expect("your drive is full. do `cargo clean`");
        *blob_id += 1;
        buf.push_str(&format!("include_bytes!({:?})", file.display()));
    }

    make(&mut buf, out_dir.clone(), out_dir, &mut id);

    buf.parse().unwrap()
}

#[proc_macro]
/// Build a wasm module. Requires `wasm32-unknown-unknown` target
/// ```rust
/// let blob = cross! {
///     "wasm32-unknown-unknown"
///     pub fn hi() -> String {
///         "hi".to_string()
///     }
/// };
/// ```
pub fn wasm(tokens: TokenStream) -> TokenStream {
    let mut id = unsafe { COMP_BLOB_ID.lock().unwrap() };

    let (dir, target) = build(tokens, Some("wasm32-unknown-unknown".to_string()));

    let out_dir = dir.join(format!("target/{target}/debug"));

    let file = out_dir.join(format!("../blob{}", *id));
    copy(out_dir.join("transarch_tmp_pkg.wasm"), &file).expect("copy failed");

    *id += 1;

    format!("include_bytes!({:?})", file.display())
        .parse()
        .unwrap()
}
