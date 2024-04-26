use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {

    // Get Working Dir from Environment Variable {CARGO_MANIFEST_DIR}
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(path) => {
            path
        },
        Err(err) => {
            println!("cargo::warning=Could not fetch CARGO_MANIFEST_DIR from the environment: {:?}", err);
            exit(-1);
        }
    };

    // Get the Source Code file for the FFI libdocx
    let ffi_library_source_file = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib/libkampfrichtereinsatzplaene_docx/libkampfrichtereinsatzplaene_docx/FFI.cs");
    let ffi_library_source_file1 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib/libkampfrichtereinsatzplaene_docx/libkampfrichtereinsatzplaene_docx/Types.cs");
    let ffi_library_source_file2 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib/libkampfrichtereinsatzplaene_docx/libkampfrichtereinsatzplaene_docx/DocumentWriter.cs");
    let ffi_library_source_file3 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib/libkampfrichtereinsatzplaene_docx/libkampfrichtereinsatzplaene_docx.sln");
    let ffi_library_source_file4 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib/libkampfrichtereinsatzplaene_docx/libkampfrichtereinsatzplaene_docx/libkampfrichtereinsatzplaene_docx.csproj");

    // Watch for changes in the FFI Source File
    println!("cargo::rerun-if-changed={}", ffi_library_source_file.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file1.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file2.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file3.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file4.display());

    // Get the main folder of the FFI libdocx library
    let ffi_library_main_dir = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib/libkampfrichtereinsatzplaene_docx");

    // Get the needed runtime identifier
    let dotnet_rid = match std::env::var("CARGO_CFG_TARGET_OS") {
        Ok(target_os) => {
            let target_os_dotnet = match target_os.as_str() {
                "windows" => { "win" },
                "macos" => { "osx" },
                "linux" => { "linux" },
                _ => {
                    println!("cargo::warning=Unknown/Unsupported target OS: {}", target_os);
                    exit(-1);
                }
            };
            let target_arch_dotnet = match std::env::var("CARGO_CFG_TARGET_ARCH") {
                Ok(target_arch) => {
                    match target_arch.as_str() {
                        "x86" => {
                            // x86 only supported on Windows!
                            if target_os_dotnet == "win" {
                                "x86"
                            } else {
                                println!("cargo::warning=32-Bit architecture (x86) is only supported on Windows, but not on target OS: {}", target_os);
                                exit(-1);
                            }
                        },
                        "x86_64" => { "x64" },
                        "arm" => {
                            // 32-Bit ARM is only supported on Linux
                            if target_os_dotnet == "linux" {
                                "arm"
                            } else {
                                println!("cargo::warning=32-Bit architecture (ARM) is only supported on Linux, but not on target OS: {}", target_os);
                                exit(-1);
                            }
                        }
                        "aarch64" => { "arm64" },
                        _ => {
                            println!("cargo::warning=Unknown/Unsupported target ARCH: {}", target_arch);
                            exit(-1);
                        }
                    }
                }
                Err(err) => {
                    println!("cargo::warning=Could not fetch CARGO_CFG_TARGET_ARCH from the environment: {:?}", err);
                    exit(-1);
                }
            };
            // Combine OS and ARCH to .NET RID
            let mut temp_str = String::from(target_os_dotnet);
            temp_str.push_str(format!("-{target_arch_dotnet}").as_str());
            temp_str
        }
        Err(err) => {
            println!("cargo::warning=Could not fetch CARGO_CFG_TARGET_OS from the environment: {:?}", err);
            exit(-1);
        }
    };

    // Compile the library natively to the current architecture
    let mut compile_command = Command::new("dotnet");
    compile_command.current_dir(ffi_library_main_dir.clone().join("libkampfrichtereinsatzplaene_docx")).args([
        "publish",
        "-r",
        &dotnet_rid,
        "-c",
        "Release",
        "-o",
        "../build"
    ]);
    match compile_command.spawn() {
        Ok(mut compiler_process) => {
            match compiler_process.wait() {
                Ok(exit_status) => {
                    if exit_status.success() {} else {
                        println!("cargo::warning=An error occured while executing the .NET compile step. Exit code was: {}", exit_status.code().unwrap());
                        exit(-1);
                    }
                }
                Err(err) => {
                    println!("cargo::warning=An error occured while executing the .NET compile step: {:?}", err);
                    exit(-1);
                }
            }
        }
        Err(err) => {
            println!("cargo::warning=An error occured while executing the .NET compile step: {:?}", err);
            exit(-1);
        }
    }

    // We got here, which means that "dotnet publish" exited correctly (exit code 0)
    // Next, check if we got a library where we expected it.
    #[cfg(target_os = "macos")]
        let build_shared_library = ffi_library_main_dir.clone().join("build/libkampfrichtereinsatzplaene_docx.dylib");
    #[cfg(target_os = "windows")]
        let build_shared_library = ffi_library_main_dir.clone().join("build/libkampfrichtereinsatzplaene_docx.dll");
    #[cfg(target_os = "linux")]
        let build_shared_library = ffi_library_main_dir.clone().join("build/libkampfrichtereinsatzplaene_docx.so");
    if !build_shared_library.exists() {
        println!("cargo::warning=The natively built library cannot be found at the following path: {}", build_shared_library.display());
        exit(-1);
    }

    // Extra Windows stuff to copy the static .lib file (which we link to at compile time)
    // DLL still needs to be available at the relevant execution path
    // WE NEED TO REMOVE THE "LIB" PART BECAUSE WINDOWS IS DUMB.
    #[cfg(target_os = "windows")]
    {
        let target_os_dotnet = match std::env::var("CARGO_CFG_TARGET_OS") {
            Ok(target_os) => {
                let target_os_dotnet = match target_os.as_str() {
                    "windows" => { "win" },
                    "macos" => { "osx" },
                    "linux" => { "linux" },
                    _ => {
                        println!("cargo::warning=Unknown/Unsupported target OS: {}", target_os);
                        exit(-1);
                    }
                };
                target_os_dotnet
            }
            Err(err) => {
                println!("cargo::warning=Could not fetch CARGO_CFG_TARGET_ARCH from the environment: {:?}", err);
                exit(-1);
            }
        };
        let target_arch_dotnet = match std::env::var("CARGO_CFG_TARGET_ARCH") {
            Ok(target_arch) => {
                match target_arch.as_str() {
                    "x86" => {
                        // x86 only supported on Windows!
                        if target_os_dotnet == "win" {
                            "x86"
                        } else {
                            println!("cargo::warning=32-Bit architecture (x86) is only supported on Windows, but not on target OS.");
                            exit(-1);
                        }
                    },
                    "x86_64" => { "x64" },
                    "arm" => {
                        // 32-Bit ARM is only supported on Linux
                        if target_os_dotnet == "linux" {
                            "arm"
                        } else {
                            println!("cargo::warning=32-Bit architecture (ARM) is only supported on Linux, but not on target OS.");
                            exit(-1);
                        }
                    }
                    "aarch64" => { "arm64" },
                    _ => {
                        println!("cargo::warning=Unknown/Unsupported target ARCH: {}", target_arch);
                        exit(-1);
                    }
                }
            }
            Err(err) => {
                println!("cargo::warning=Could not fetch CARGO_CFG_TARGET_ARCH from the environment: {:?}", err);
                exit(-1);
            }
        };
        let win_static_library = ffi_library_main_dir.clone().join(format!("libkampfrichtereinsatzplaene_docx/bin/{target_arch_dotnet}/Release/net8.0/{dotnet_rid}/native/libkampfrichtereinsatzplaene_docx.lib"));
        std::fs::copy(win_static_library, ffi_library_main_dir.clone().join("build/kampfrichtereinsatzplaene_docx.lib")).unwrap();
        std::fs::copy(build_shared_library, ffi_library_main_dir.clone().join("build/kampfrichtereinsatzplaene_docx.dll")).unwrap();
    }
    
    // We finally have everything. God bless us. Let's set the linker flags.
    let build_shared_library_dir = ffi_library_main_dir.clone().join("build");
    println!("cargo:rustc-link-search=native={}", build_shared_library_dir.display());

    // Build Tauri normally
    tauri_build::build();
}
