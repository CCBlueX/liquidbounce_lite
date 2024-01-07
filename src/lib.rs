use anyhow::Result;
use jni::JavaVM;
use jni::sys::{jsize, jint};

use windows::{ 
    Win32::Foundation::*, 
    Win32::System::SystemServices::*, 
    Win32::System::Console::{AllocConsole, FreeConsole},
    Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress},
};
use windows::core::s;

use tracing::{info, Level, error, trace, debug};
use tracing_subscriber::FmtSubscriber;

use std::thread;

// Define a type alias for the `JNI_GetCreatedJavaVMs` function pointer.
#[allow(non_camel_case_types)]
type JNI_GetCreatedJavaVMs_Fn = fn(vm_buf: *mut *mut JavaVM, buf_len: jsize, num_vms: *mut jsize) -> jint;

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
    if let Err(e) = unsafe { AllocConsole() } {
        error!("Failed to allocate console: {:?}", e);
    }

    // Attempt to start the client and log any error that occurs.
    if let Err(e) = start_client() {
        error!("Failed to start the client: {:?}", e);
    }

    // Clean up by freeing the allocated console when done.
    let _ = unsafe { FreeConsole() };
}

// Responsible for starting the client, including attaching to the JVM and initializing logging.
pub fn start_client() -> Result<()> {
    // Attempt to obtain a module handle for the JVM DLL and log if successful.
    let jvm_dll_handle = unsafe { GetModuleHandleA(s!("jvm.dll")) }?;
    trace!("Obtained jvm.dll handle: {:?}", jvm_dll_handle);

    // Retrieve the `JNI_GetCreatedJavaVMs` function from the JVM DLL.
    let jni_get_created_java_vms_fn_ptr = unsafe { GetProcAddress(jvm_dll_handle, s!("JNI_GetCreatedJavaVMs")) };
    let get_created_java_vms: JNI_GetCreatedJavaVMs_Fn = unsafe { std::mem::transmute(jni_get_created_java_vms_fn_ptr) };
    debug!("Address of JNI_GetCreatedJavaVMs function: {:?}", jni_get_created_java_vms_fn_ptr);

    // Initialize variables to hold the JVM instance and JVM count.
    debug!("Retrieving the list of JVMs.");
    let mut jvm_instance: *mut JavaVM = std::ptr::null_mut();
    let mut number_of_jvms: jsize = 0;
    let get_jvms_result = get_created_java_vms(&mut jvm_instance as _, 1, &mut number_of_jvms as _);
    debug!("Number of JVMs found: {}", number_of_jvms);
    trace!("JNI_GetCreatedJavaVMs response code: {:?}", get_jvms_result);

    // Convert the raw JVM pointer to a `JavaVM` instance and attach the current thread as a daemon.
    let jvm = unsafe { JavaVM::from_raw(jvm_instance as _)? };
    jvm.attach_current_thread_as_daemon()?;

    // Retrieve the Java environment for further operations.
    let mut jvm_environment = jvm.get_env()?;
    info!("Obtained the JVM environment.");

    // TODO: Implement a name remapper for working with obfuscated code.
    // TODO: Generate an SDK based on source code or mappings.

    // Look up the Minecraft client class using the JNI environment.
    let minecraft_client_class = jvm_environment.find_class("net/minecraft/client/MinecraftClient")?;
    trace!("Located MinecraftClient class: {:?}", minecraft_client_class);

    // Retrieve the field ID for the static `instance` field of the Minecraft client class.
    let minecraft_client_instance_field_id = jvm_environment.get_static_field_id(minecraft_client_class, "instance", "Lnet/minecraft/client/MinecraftClient;")?;
    trace!("MinecraftClient `instance` field ID: {:?}", minecraft_client_instance_field_id);

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