use std::{env, mem};
use std::fs::File;
use std::os::fd::AsRawFd;
use chrono::{Datelike, Timelike};
use crate::types::ApplicationError;

pub fn activateLogging() -> Result<(), ApplicationError> {

    let stdout_fd = std::io::stdout().as_raw_fd();
    let stderr_fd = std::io::stderr().as_raw_fd();

    let time_and_date = chrono::Local::now();
    let time_and_date_string = format!["{}-{}-{}_{}-{}-{}", time_and_date.year(), time_and_date.month(), time_and_date.day(), time_and_date.time().hour(), time_and_date.time().minute(), time_and_date.time().second()];

    let stdout_file;
    let stderr_file;

    #[cfg(not(target_os = "windows"))]
    // Get Program Directory at Runtime
    match env::current_exe() {
        Ok(exe_path) => {
            let parent_folder = exe_path.parent().unwrap().parent().unwrap().to_path_buf();
            let log_folder = parent_folder.join("Logs");
            if !log_folder.exists() {
                match std::fs::create_dir_all(log_folder.clone()) {
                    Ok(()) => {}
                    Err(e) => panic!("Could not create the Resource folder: {e}"),
                };
            }
            // Create file for stdout
            stdout_file = match File::create(log_folder.join(format!["LOG__{}__STDOUT.txt", time_and_date_string.clone()])) {
                Ok(file) => {file}
                Err(err) => {
                    println!("{:?}", err);
                    return Err(ApplicationError::FailedToCreateStdOutFileError);
                }
            };
            // Create file for stdout
            stderr_file = match File::create(log_folder.join(format!["LOG__{}__STDERR.txt", time_and_date_string.clone()])) {
                Ok(file) => {file}
                Err(err) => {
                    println!("{:?}", err);
                    return Err(ApplicationError::FailedToCreateStdErrFileError);
                }
            };
        },
        Err(e) => panic!("Could not get the current executable path: {e}"),
    };

    // First, get the file descriptors
    let stdout_file_fd = stdout_file.as_raw_fd();
    let stderr_file_fd = stderr_file.as_raw_fd();

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
    return Ok(());
}