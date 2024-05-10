// Until incrementing the version number works properly
#![allow(unused_imports)]

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
use vergen::EmitBuilder;

fn main() {

    // Let vergen do its thing
    EmitBuilder::builder()
        .all_build()
        .git_sha(true)
        .git_branch()
        .all_cargo()
        .all_rustc()
        .all_sysinfo()
        .fail_on_error()
        .emit()
        .unwrap();

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
    
    // CURRENTLY BUGGY. CAUSES INSTANT REBUILD WITH RUST_ANALYZER.
    // DEACTIVATE FOR NOW.
    // Increment build number, if we are not on a release!
    // THIS IS THE WORST BTW. I AM SO SORRY.
    /*
    let cargo_toml_path = PathBuf::from(manifest_dir.clone()).join("Cargo.toml");
    if cargo_toml_path.exists() && cargo_toml_path.is_file() {
        let cargo_toml_file = match File::options().read(true).open(cargo_toml_path.clone()) {
            Ok(file) => file,
            Err(err) =>  {
                println!("cargo::warning=Could not open Cargo.toml file: {:?}", err);
                exit(-1);
            }
        };
        let cargo_toml_buffer = BufReader::new(cargo_toml_file);
        let cargo_toml_lines: Vec<String> = cargo_toml_buffer.lines().map(|line| line.unwrap()).collect();
        let mut new_line_vec = vec![];
        for line in &cargo_toml_lines {
            if line.contains("version =") && !line.contains("{ version =") {
                let substrings = line.split_once('"').unwrap();
                // If this fails, this is a regular release build
                // We just add the line and continue.
                let version_substrings = match substrings.1.split_once('-') {
                    Some(substr) => substr,
                    None => {
                        let mut line_with_newline = line.clone();
                        line_with_newline.push_str("\n");
                        new_line_vec.push(line_with_newline);
                        continue;
                    },
                };
                let version_triple = version_substrings.0;
                let prerelease_substrings = version_substrings.1.split_once('-').unwrap();
                let prerelease_word = prerelease_substrings.0;
                let prerelease_no = prerelease_substrings.1.strip_suffix('"').unwrap();
                let mut build_no = prerelease_no.parse::<i64>().unwrap();
                build_no += 1;
                new_line_vec.push(format!(r#"version = "{}-{}-{}"{}"#, version_triple, prerelease_word, build_no.to_string(), "\n"));
            } else {
                let mut line_with_newline = line.clone();
                line_with_newline.push_str("\n");
                new_line_vec.push(line_with_newline);
            }
        }
        // WTF happened here. This is worse than bad. NGL.
        let mut byte_vec = vec![];
        for new_line in new_line_vec {
            for byte in new_line.as_bytes() {
                byte_vec.push(*byte);
            }
        }
        let mut cargo_toml_file2 = match File::options().read(true).write(true).truncate(true).open(cargo_toml_path) {
            Ok(file) => file,
            Err(err) =>  {
                println!("cargo::warning=Could not open Cargo.toml file: {:?}", err);
                exit(-1);
            }
        };
        cargo_toml_file2.write_all(byte_vec.as_slice()).unwrap();
    }
    */

    // Get the Source Code file for the FFI libdocx
    let ffi_library_source_file = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib").join("libkampfrichtereinsatzplaene_docx").join("libkampfrichtereinsatzplaene_docx").join("FFI.cs");
    let ffi_library_source_file1 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib").join("libkampfrichtereinsatzplaene_docx").join("libkampfrichtereinsatzplaene_docx").join("Types.cs");
    let ffi_library_source_file2 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib").join("libkampfrichtereinsatzplaene_docx").join("libkampfrichtereinsatzplaene_docx").join("DocumentWriter.cs");
    let ffi_library_source_file3 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib").join("libkampfrichtereinsatzplaene_docx").join("libkampfrichtereinsatzplaene_docx.sln");
    let ffi_library_source_file4 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib").join("libkampfrichtereinsatzplaene_docx").join("libkampfrichtereinsatzplaene_docx").join("libkampfrichtereinsatzplaene_docx.csproj");
    let ffi_library_source_file5 = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib").join("libkampfrichtereinsatzplaene_docx").join("libkampfrichtereinsatzplaene_docx").join("PDFWriter.cs");

    // Watch for changes in the FFI Source File
    println!("cargo::rerun-if-changed={}", ffi_library_source_file.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file1.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file2.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file3.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file4.display());
    println!("cargo::rerun-if-changed={}", ffi_library_source_file5.display());

    // Get the main folder of the FFI libdocx library
    let ffi_library_main_dir = PathBuf::from(manifest_dir.clone()).parent().unwrap().join("lib").join("libkampfrichtereinsatzplaene_docx");

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
        let build_shared_library = ffi_library_main_dir.clone().join("build").join("libkampfrichtereinsatzplaene_docx.dylib");
    #[cfg(target_os = "windows")]
        let build_shared_library = ffi_library_main_dir.clone().join("build").join("libkampfrichtereinsatzplaene_docx.dll");
    #[cfg(target_os = "linux")]
        let build_shared_library = ffi_library_main_dir.clone().join("build").join("libkampfrichtereinsatzplaene_docx.so");
    if !build_shared_library.exists() {
        println!("cargo::warning=The natively built library cannot be found at the following path: {}", build_shared_library.display());
        exit(-1);
    }

    // Extra Windows stuff to copy the static .lib file (which we link to at compile time)
    // DLL still needs to be available at the relevant execution path
    // WE NEED TO REMOVE THE "LIB" PART BECAUSE WINDOWS IS DUMB.
    #[cfg(target_os = "windows")]
    {
        let win_static_library_candidates = match glob::glob(format!("{}/**/libkampfrichtereinsatzplaene_docx.lib", ffi_library_main_dir.clone().join("libkampfrichtereinsatzplaene_docx").join("bin").to_str().unwrap()).as_str()) {
            Ok(glob) => {glob}
            Err(err) => {
                println!("cargo::warning=Glob threw an error while finding the windows static library file: {:?}", err);
                exit(-1);
            }
        };
        for candidate in win_static_library_candidates {
            if candidate.is_ok() {
                std::fs::copy(candidate.unwrap(), ffi_library_main_dir.clone().join("build").join("kampfrichtereinsatzplaene_docx.lib")).unwrap();
                break;
            }
        }
    }

    // If we are on Linux, add the relevant directory to the rpath
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/SharedLibs");

    // We finally have everything. God bless us. Let's set the linker flags.
    let build_shared_library_dir = ffi_library_main_dir.clone().join("build");
    println!("cargo:rustc-link-search=native={}", build_shared_library_dir.to_str().unwrap());

    // Build Tauri normally
    tauri_build::build();
}
