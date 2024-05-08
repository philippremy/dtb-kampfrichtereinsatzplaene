use std::mem;
use std::fs::File;
use chrono::{Datelike, Timelike};
use crate::types::ApplicationError;

#[cfg(target_os = "windows")]
use windows::Win32::System::Console::{SetStdHandle, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE};
#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawHandle;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HANDLE;

#[cfg(not(target_os = "windows"))]
use std::os::fd::AsRawFd;
use crate::{STDERR_FILE, STDOUT_FILE};

pub fn activateLogging() -> Result<(), ApplicationError> {

    let time_and_date = chrono::Local::now();
    let time_and_date_string = format!["{}-{}-{}_{}-{}-{}", time_and_date.year(), time_and_date.month(), time_and_date.day(), time_and_date.time().hour(), time_and_date.time().minute(), time_and_date.time().second()];

    let stdout_file;
    let stderr_file;

    let mut stdout_file_name = format!["LOG__{}__STDOUT.txt", time_and_date_string.clone()];
    let mut stderr_file_name = format!["LOG__{}__STDERR.txt", time_and_date_string.clone()];

    // Severe permission issues on Windows when using the approach above. Windows has unique folders to store application data.
    // Program Directory is not the place for that.
    match directories::BaseDirs::new() {
        None => {panic!("Could not get the Windows Base Dirs. Important files will be missing and we cannot get them from anywhere else, so we exit here.")}
        Some(dirs) => {
            let appdata_roaming_dir = dirs.data_dir();
            let application_log_dir = appdata_roaming_dir.join("de.philippremy.dtb-kampfrichtereinsatzplaene").join("Logs");
            // Create the folder if it does not exist!
            match std::fs::create_dir_all(application_log_dir.clone()) {
                Ok(()) => {}
                Err(err) => {
                    panic!("Could not create the AppData dir: {:?}", err);
                }
            }
            // Create file for stdout
            stdout_file = match File::create(application_log_dir.join(stdout_file_name.clone())) {
                Ok(file) => {file}
                Err(err) => {
                    println!("{:?}", err);
                    return Err(ApplicationError::FailedToCreateStdOutFileError);
                }
            };
            // Create file for stdout
            stderr_file = match File::create(application_log_dir.join(stderr_file_name.clone())) {
                Ok(file) => {file}
                Err(err) => {
                    println!("{:?}", err);
                    return Err(ApplicationError::FailedToCreateStdErrFileError);
                }
            };

            // Overwrite the file names (now an absolute path)!
            stdout_file_name = application_log_dir.join(stdout_file_name).to_str().unwrap().to_string();
            stderr_file_name = application_log_dir.join(stderr_file_name).to_str().unwrap().to_string();

        }
    }

    // Use file descriptors on Unix
    #[cfg(not(target_os = "windows"))]
    {
        // First, get the file descriptors
        let stdout_file_fd = stdout_file.as_raw_fd();
        let stderr_file_fd = stderr_file.as_raw_fd();
        let stdout_fd = std::io::stdout().as_raw_fd();
        let stderr_fd = std::io::stderr().as_raw_fd();

        // Forget about the files, so they don't get deallocated!
        // They have to be available until the end of the program.
        mem::forget(stdout_file);
        mem::forget(stderr_file);

        // Now change the file handles and call it day.
        unsafe {
            let result_stdout = libc::dup2(stdout_file_fd, stdout_fd);
            if result_stdout == -1 {
                println!("errno: {:?}", std::io::Error::last_os_error());
                return Err(ApplicationError::LibcDup2StdOutError);
            }
            let result_stderr = libc::dup2(stderr_file_fd, stderr_fd);
            if result_stderr == -1 {
                println!("errno: {:?}", std::io::Error::last_os_error());
                return Err(ApplicationError::LibcDup2StdErrError);
            }
        }
    }

    // Use file handles on Windows
    #[cfg(target_os = "windows")]
    unsafe {
        // First, get the file handles
        let stdout_file_fh = stdout_file.as_raw_handle();
        let stderr_file_fh = stderr_file.as_raw_handle();

        // Forget about the files, so they don't get deallocated!
        // They have to be available until the end of the program.
        mem::forget(stdout_file);
        mem::forget(stderr_file);

        // Now change the file handles and call it day.
        match SetStdHandle(STD_OUTPUT_HANDLE, HANDLE(stdout_file_fh as isize)) {
            Ok(()) => {},
            Err(err) => {
                println!("errno: {:?}", err);
                return Err(ApplicationError::LibcDup2StdOutError);
            }
        }
        match SetStdHandle(STD_ERROR_HANDLE, HANDLE(stderr_file_fh as isize)) {
            Ok(()) => {},
            Err(err) => {
                println!("errno: {:?}", err);
                return Err(ApplicationError::LibcDup2StdErrError);
            }
        }
    }

    // If everything succeeded, we should set the paths to our global statics
    // SAFETY: The statics get only accessed by the mail program, which is invoked much later.
    // We can guarantee that there is no race condition as it will only be written once (might
    // even not be true)
    unsafe {
        STDOUT_FILE = Some(stdout_file_name.clone());
        STDERR_FILE = Some(stderr_file_name.clone());
    }

    return Ok(());
}