use std::ffi::{c_char, CString};
use std::path::PathBuf;
use crate::types::{ApplicationError, Storage};

/// Extern functions from libkampfrichtereinsatzplan_docx
/// Can only be called in an unsafe context.
/// Part of the global FFI module.
#[link(name = "kampfrichtereinsatzplaene_docx", kind = "dylib")]
extern "C" {
    /// Stub function
    #[allow(dead_code)]
    fn stub_func();
    /// Function to create the plans from data
    /// Param 1: Const Pointer to a prepared FFIStorage struct containing the raw data.
    /// Param 2: Const Pointer to a c_char containing the path where the plan should be saved to.
    /// Returns: A FFIError.
    fn ffi_create_from_raw_data(json_data: *const c_char, save_path: *const c_char) -> ApplicationError;
}

/// To be called by the main application, saves the plans to the hard drive.
/// Param 1: A immutable reference to the global storage struct.
/// Param 2: A PathBuf containing the path where the plan should be saved to.
/// Returns: A Result either containing void (() == Success!) or an FFIError which gives more information.
pub fn create_tables_docx(storage: &Storage, save_path: PathBuf) -> Result<(), ApplicationError> {

    // Serialize data and get pointer to it
    let serialized_data = match serde_json::to_string(storage) {
        Ok(data) => {data}
        Err(_err) => { return Err(ApplicationError::JSONSerializeError) }
    };
    let serialized_data_cstring = match CString::new(serialized_data) {
        Ok(cstr) => {cstr}
        Err(_err) => { return Err(ApplicationError::CStringNullError) }
    };
    let serialized_data_cstring_ptr = serialized_data_cstring.as_ptr();

    // Get pointer to save path
    let save_path_cstring = match CString::new(save_path.to_str().unwrap()) {
        Ok(cstr) => {cstr}
        Err(_err) => { return Err(ApplicationError::CStringNullError) }
    };
    let save_path_cstring_ptr = save_path_cstring.as_ptr();

    // Call function and retrieve Result
    unsafe {
        let error_code = ffi_create_from_raw_data(serialized_data_cstring_ptr, save_path_cstring_ptr);
        assert_eq!(error_code, ApplicationError::NoError);
    }

    return Ok(());
}