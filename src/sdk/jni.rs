use anyhow::{Result, anyhow};
use jni::{JavaVM, JNIEnv};
use jni::sys::{jsize, jint};
use derive_new::new;
use windows::{ 
    Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress},
};
use windows::core::s;

use tracing::{info, error, trace, debug};

// Define a type alias for the `JNI_GetCreatedJavaVMs` function pointer.
#[allow(non_camel_case_types)]
type JNI_GetCreatedJavaVMs_Fn = fn(vm_buf: *mut *mut JavaVM, buf_len: jsize, num_vms: *mut jsize) -> jint;

// Responsible for starting the client, including attaching to the JVM and initializing logging.
pub fn retrieve_java_vm<'a>() -> Result<JavaVM> {
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

    return Ok(jvm);
}

