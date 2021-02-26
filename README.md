# phys-rs

Simple physics engine written in Rust and can compiling to Wasm. Library created for small pet project.

## Compiling

To compile WASM files need add Rust target by command:
```bash
rustup target add wasm32-unknown-unknown
```
and install wasm-pack from
```bash
https://rustwasm.github.io/wasm-pack/installer/
```
Then build with target param pass:
```bash
wasm-pack build --release --target web
```
If need pass --out-dir param to build in specified directory.