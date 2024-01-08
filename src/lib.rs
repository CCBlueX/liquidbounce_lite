use anyhow::Result;

use sdk::{jni::retrieve_java_vm, game::MinecraftClient};
use windows::{ 
    Win32::Foundation::*, 
    Win32::System::SystemServices::*, 
    Win32::System::Console::{AllocConsole, FreeConsole, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE}
};

use tracing::{info, Level, error, trace, debug};
use tracing_subscriber::FmtSubscriber;

use std::{thread::{self, sleep}, time::Duration, os::windows::io::AsRawHandle};

pub mod sdk;

// The entry function responsible for the primary execution thread of the application.
pub fn main_thread() {
    // Setup logging with the `tracing` crate to provide structured, level-based logging.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting the default subscriber failed");

    // Initialize a Windows console using the Win32 API
    // Attempt to allocate a console and log any error that occurs.
    //if let Err(e) = alloc_console() {
    //    error!("Failed to allocate console: {:?}", e);
    //}

    // Attempt to start the client and log any error that occurs.
    if let Err(e) = start_client() {
        error!("Failed to start the client: {:?}", e);
    }

    // Clean up by freeing the allocated console when done.
    //let _ = unsafe { FreeConsole() };
}

pub fn alloc_console() -> Result<()> {
    unsafe { AllocConsole() }?;

    // Hook the standard output streams to the console.
    let stdout = std::io::stdout();
    let handle = stdout.lock().as_raw_handle();
    let handle = HANDLE(handle as isize);
    unsafe { SetStdHandle(STD_OUTPUT_HANDLE, handle) }?;

    let stderr = std::io::stderr();
    let handle = stderr.lock().as_raw_handle();
    let handle = HANDLE(handle as isize);
    unsafe { SetStdHandle(STD_ERROR_HANDLE, handle) }?;

    Ok(())
}

// Responsible for starting the client, including attaching to the JVM and initializing logging.
pub fn start_client() -> Result<()> {
    let jvm = retrieve_java_vm()?;

    loop {
        // Retrieve the Java environment for further operations.
        let jni_env = jvm.get_env()?;
        let client = MinecraftClient::get_instance(jni_env)?;
        if let Err(e) = report_position(client) {
            error!("Failed to report position: {:?}", e);
        }

        sleep(Duration::from_secs(10));
    }

    Ok(())
}

pub fn report_position(mut client: MinecraftClient) -> Result<()> {
    let mut player = client.get_player()?;

    let x = player.get_x()?;
    let y = player.get_y()?;
    let z = player.get_z()?;
    info!("Player position: ({}, {}, {})", x, y, z);

    Ok(())
}

// The DLL entry point, which is executed when the DLL is loaded or unloaded.
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    reserved: *mut ())
    -> bool
{
    match call_reason {
        // When the DLL is attached, spawn a new thread to run the `main_thread` function.
        DLL_PROCESS_ATTACH => {
            thread::spawn(main_thread);
        },
        // No action is taken when the DLL is detached.
        DLL_PROCESS_DETACH => {},
        _ => {}
    }

    // Return true to indicate successful handling of the DLL entry point.
    true
}