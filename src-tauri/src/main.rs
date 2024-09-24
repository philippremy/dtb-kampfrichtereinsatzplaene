// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_doc_comments)]
#![allow(non_snake_case)]
#![warn(clippy::undocumented_unsafe_blocks)]

use crate::types::{
    ApplicationError, FrontendStorage, Storage, UpdateAvailablePayload, UpdateProgressPayload,
};
use crate::MailImpl::{send_mail, MessageKind};
use crate::FFI::{create_tables_docx, create_tables_pdf};
use tauri::menu::{AboutMetadataBuilder, Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri_plugin_updater::UpdaterExt;
use std::collections::HashMap;
use std::fs::File;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::process::{abort, Command};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
#[cfg(target_os = "linux")]
use std::sync::Mutex;
#[cfg(target_os = "linux")]
use std::time::Duration;
use std::{env, thread};
use tauri::{AppHandle, Emitter, Manager, State, Wry};
use tokio::time::sleep;

mod FFI;
mod MailImpl;
mod log;
mod types;
mod PrintToPdfImpl;

// Linux struct
#[cfg(target_os = "linux")]
pub struct DbusState(Mutex<Option<dbus::blocking::SyncConnection>>);

// Force PlatformWebview to be Send
struct PlatformWebViewWrapper {
    inner: Option<tauri::webview::PlatformWebview>
}
unsafe impl Send for PlatformWebViewWrapper {}

// Statics
static mut SAVE_PATH: Option<String> = None;
static LLVM_VER: &'static str = env!("VERGEN_RUSTC_LLVM_VERSION");
static TARGET_TRIPLE: &'static str = env!("VERGEN_CARGO_TARGET_TRIPLE");
static GIT_COMMIT: &'static str = env!("VERGEN_GIT_SHA");
static GIT_BRANCH: &'static str = env!("VERGEN_GIT_BRANCH");
static APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
static mut STDOUT_FILE: Option<String> = None;
static mut STDERR_FILE: Option<String> = None;
static mut MAINWINDOW_LOADED: AtomicBool = AtomicBool::new(false);
static mut UPDATE_REQUESTED: AtomicI8 = AtomicI8::new(0);
static mut UPDATE_PROGRESS: AtomicU64 = AtomicU64::new(0);
static PLATFORM_WEBVIEW: LazyLock<Arc<tokio::sync::Mutex<PlatformWebViewWrapper>>> = LazyLock::new(|| {
    Arc::new(
        tokio::sync::Mutex::new(
            PlatformWebViewWrapper {
                inner: None
            }
        )
    )
});

#[tauri::command]
fn update_mainwindow_loading_state(visible: bool) {
    // SAFETY: This is safe, we are only using atomic operations
    unsafe {
        MAINWINDOW_LOADED.store(visible, Ordering::Relaxed);
    }
}

// MARK: Func: Update Storage Data
/// Function to update the global storage from the frontend.
/// Param 1: The frontend storage struct provided by Javascript
/// Param 2: The managed Storage state object provided by Tauri.
/// Returns: An ApplicationError to be handled by the frontend.
#[tauri::command]
fn update_storage_data(
    frontend_storage: FrontendStorage,
    storage: State<Storage>,
) -> ApplicationError {
    // Update all data. First, lock the relevant object and then update it.
    // Immediately after drop the lock.

    // For wk_name
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_name;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_date
    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_date;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_responsible_person
    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_responsible_person;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_judgesmeeting_time
    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_judgesmeeting_time;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_replacement_judges
    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_replacement_judges.unwrap();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_judgingtables
    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_judgingtables.unwrap();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return ApplicationError::MutexPoisonedError;
        }
    };

    // No error occured, return NoError variant.
    return ApplicationError::NoError;
}

// MARK: Func: Create Wettkampf Window
/// Tauri Command for creating a window that creates a new Wettkampf
#[tauri::command]
async fn create_wettkampf(app_handle: AppHandle) -> ApplicationError {

    let create_wettkampf_window = match tauri::WebviewWindowBuilder::new(
        &app_handle,
        "createWettkampf",
        tauri::WebviewUrl::App(PathBuf::from("createWettkampf.html"))
    )
    .inner_size(515.0, 600.0)
    .title("Wettkampf erstellen")
    .focused(true)
    .menu(build_menus(MenuKind::CreateWettkampf, &app_handle))
    .center()
    .build()
    {
        Ok(window) => window,
        Err(err) => {
            eprintln!("Could not build CreateWettkampf window: {:?}", err);
            return ApplicationError::TauriWindowCreationError;
        }
    };
    match create_wettkampf_window.show() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Could not show CreateWettkampf window: {:?}", err);
            return ApplicationError::TauriWindowShowError;
        }
    }
    return ApplicationError::NoError;
}

// MARK: Func: Sync WK Data and open Editor
/// Syncs the initial WK data and hands off to the GUI WK Editor.
/// Param 1: FrontendStorage data struct (provided by the Frontend) through serde::Deserialize
/// Param 2: State<'_ Storage> global storage provided by Tauri.
/// Returns: A Result always containing an Ok(ApplicationError) value - never void or something went terribly wrong.
/// CAVEATS: async functions cannot simply use borrowed data like State<T>, so we need the anonymous lifetime specifier "'_" and have to return a Result.
#[tauri::command]
async fn sync_wk_data_and_open_editor(
    data: FrontendStorage,
    storage: State<'_, Storage>,
    app_handle: AppHandle,
) -> Result<ApplicationError, ()> {
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = data.wk_name.clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = data.wk_place;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = data.wk_date;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = data.wk_judgesmeeting_time;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = data.wk_responsible_person;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    // Create the Editor Window
    let editor_window = match tauri::WebviewWindowBuilder::new(
        &app_handle,
        "editor",
        tauri::WebviewUrl::App(PathBuf::from("editor.html")),
    )
    .inner_size(1250.0, 800.0)
    .title(format!["{} (nicht gespeichert)", data.wk_name])
    .focused(true)
    .menu(build_menus(MenuKind::Editor, &app_handle))
    .center()
    .build()
    {
        Ok(window) => window,
        Err(err) => {
            eprintln!("Could not build the Editor window: {:?}", err);
            return Ok(ApplicationError::TauriWindowCreationError);
        }
    };
    match editor_window.show() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Could not show the Editor window: {:?}", err);
            return Ok(ApplicationError::TauriWindowShowError);
        }
    }

    return Ok(ApplicationError::NoError);
}

#[tauri::command]
async fn get_wk_data_to_frontend(
    storage: State<'_, Storage>,
) -> Result<(FrontendStorage, Option<String>), ApplicationError> {
    let mut frontend_storage = FrontendStorage::default();
    match storage.wk_name.lock() {
        Ok(guard) => {
            frontend_storage.wk_name = (*guard).clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Err(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_place.lock() {
        Ok(guard) => {
            frontend_storage.wk_place = (*guard).clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Err(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_date.lock() {
        Ok(guard) => {
            frontend_storage.wk_date = (*guard).clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Err(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(guard) => {
            frontend_storage.wk_judgesmeeting_time = (*guard).clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Err(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_responsible_person.lock() {
        Ok(guard) => {
            frontend_storage.wk_responsible_person = (*guard).clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Err(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_replacement_judges.lock() {
        Ok(guard) => {
            if guard.is_empty() {
                frontend_storage.wk_replacement_judges = Some(Vec::new());
            } else {
                frontend_storage.wk_replacement_judges = Some((*guard).clone());
            }
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Err(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgingtables.lock() {
        Ok(guard) => {
            if guard.is_empty() {
                frontend_storage.wk_judgingtables = Some(HashMap::new());
            } else {
                frontend_storage.wk_judgingtables = Some((*guard).clone());
            }
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Err(ApplicationError::MutexPoisonedError);
        }
    }

    unsafe { return Ok((frontend_storage, SAVE_PATH.clone())) };
}

#[tauri::command]
async fn sync_to_backend_and_save(
    frontendstorage: FrontendStorage,
    filepath: String,
    storage: State<'_, Storage>,
) -> Result<ApplicationError, ()> {
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_name.clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_place;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_date;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_judgesmeeting_time;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_responsible_person;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_replacement_judges {
                Some(map) => map,
                None => Vec::new(),
            };
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_judgingtables {
                Some(map) => map,
                None => HashMap::new(),
            };
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    // Serialize data
    let serialized_data = match serde_json::to_string(storage.inner()) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to serialize storage data: {:?}", err);
            return Ok(ApplicationError::JSONSerializeError);
        }
    };

    // Write file at path!
    match std::fs::write(filepath, serialized_data) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Failed to write serialized wk data to file: {:?}", err);
            return Ok(ApplicationError::RustWriteFileError);
        }
    }

    return Ok(ApplicationError::NoError);
}

/// Function to sync all stuff and create the plans using FFI
#[tauri::command]
async fn sync_to_backend_and_create_docx(
    frontendstorage: FrontendStorage,
    filepath: String,
    storage: State<'_, Storage>,
) -> Result<ApplicationError, ()> {
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_name.clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_place;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_date;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_judgesmeeting_time;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_responsible_person;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_replacement_judges {
                Some(map) => map,
                None => Vec::new(),
            };
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_judgingtables {
                Some(map) => map,
                None => HashMap::new(),
            };
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    return Ok(create_tables_docx(storage.inner(), PathBuf::from(filepath)).unwrap());
}

#[tauri::command]
async fn sync_to_backend_and_create_pdf(
    frontendstorage: FrontendStorage,
    filepath: String,
    storage: State<'_, Storage>,
) -> Result<ApplicationError, ()> {
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_name.clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_place;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_date;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_judgesmeeting_time;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_responsible_person;
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_replacement_judges {
                Some(map) => map,
                None => Vec::new(),
            };
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_judgingtables {
                Some(map) => map,
                None => HashMap::new(),
            };
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    // Process backend library docx and html
    let library_code = create_tables_pdf(storage.inner(), PathBuf::from(filepath.clone())).unwrap();

    // Check if this succeeded.
    if library_code != ApplicationError::NoError {
        return Ok(library_code);
    }

    // Fetch the generated html file
    let generated_html = filepath.clone().replace(".pdf", "_temp.html");
    let generated_docx = filepath.clone().replace(".pdf", "_temp.docx");

    // Get the shared PDF View
    let platform_webview = PLATFORM_WEBVIEW.lock().await;
    let errno = platform_webview.print_pdf(PathBuf::from(generated_html.clone()), PathBuf::from(filepath.clone()));

    // Delete temporary files
    match std::fs::remove_file(generated_html) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Could not remove temporary generated HTML file: {:?}", err);
            return Ok(ApplicationError::RemovalOfTemporaryGeneratedFilesFailed);
        }
    }

    // Delete temporary files
    match std::fs::remove_file(generated_docx) {
        Ok(()) => {}
        Err(err) => {
            println!("Could not remove temporary generated DOCX file: {:?}", err);
            return Ok(ApplicationError::RemovalOfTemporaryGeneratedFilesFailed);
        }
    }

    if errno != ApplicationError::NoError {
        return Ok(errno);
    }

    return Ok(ApplicationError::NoError);
}

// Function for loading a file from disk and importing this into frontend storage
// Then open the editor
#[tauri::command]
async fn import_wk_file_and_open_editor(
    filepath: String,
    storage: State<'_, Storage>,
    app_handle: AppHandle,
) -> Result<ApplicationError, ()> {
    // Set the static thing so we know where this was saved!
    unsafe { SAVE_PATH = Some(filepath.clone()) };

    // Deserialize the file
    let imported_storage: Storage = match serde_json::from_reader(File::open(filepath).unwrap()) {
        Ok(storage) => storage,
        Err(err) => {
            eprintln!("Could not deserialize the wk data from the imported file (Maybe the file is corrupt or invalid?): {:?}", err);
            return Ok(ApplicationError::JSONDeserializeImporterError);
        }
    };

    // Update the storage
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_name.lock().unwrap().clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_place.lock().unwrap().clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_date.lock().unwrap().clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = imported_storage
                .wk_judgesmeeting_time
                .lock()
                .unwrap()
                .clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = imported_storage
                .wk_responsible_person
                .lock()
                .unwrap()
                .clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = imported_storage
                .wk_replacement_judges
                .lock()
                .unwrap()
                .clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_judgingtables.lock().unwrap().clone();
            drop(guard);
        }
        Err(err) => {
            eprintln!("Failed to acquire lock of mutex: {:?}", err);
            return Ok(ApplicationError::MutexPoisonedError);
        }
    }

    // Open the Editor!
    let editor_window = match tauri::WebviewWindowBuilder::new(
        &app_handle,
        "editor",
        tauri::WebviewUrl::App(PathBuf::from("editor.html")),
    )
    .inner_size(1250.0, 800.0)
    .title(format![
        "{} (gespeichert)",
        imported_storage.wk_name.lock().unwrap()
    ])
    .focused(true)
    .menu(build_menus(MenuKind::Editor, &app_handle))
    .center()
    .build()
    {
        Ok(window) => window,
        Err(err) => {
            eprintln!("Could not build the Editor window: {:?}", err);
            return Ok(ApplicationError::TauriWindowCreationError);
        }
    };
    match editor_window.show() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Could not show the Editor window: {:?}", err);
            return Ok(ApplicationError::TauriWindowShowError);
        }
    }

    return Ok(ApplicationError::NoError);
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
fn show_item_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &path]) // The comma after select is not a typo
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        let path_buf = PathBuf::from(&path);
        if path_buf.is_dir() {
            Command::new("open")
                .args([&path])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("open")
                .args(["-R", &path])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn open_bug_reporter(kind: String, app_handle: AppHandle) -> Result<ApplicationError, ()> {
    // If we already found the window, show it. Then we should not create a new one or we might
    // panic.
    if app_handle.windows().contains_key("bugReport") {
        let windows = app_handle.windows();
        let bug_report_windows = match windows.get("bugReport") {
            Some(window) => window,
            None => {
                eprintln!("Failed to get the bugReporter window from HashMap, although it was expected to be present.");
                return Ok(ApplicationError::TauriExistingWindowNotFoundError);
            }
        };
        match bug_report_windows.show() {
            Ok(()) => return Ok(ApplicationError::NoError),
            Err(err) => {
                eprintln!("Failed to show BugReport Window: {:?}", err);
                return Ok(ApplicationError::TauriWindowShowError);
            }
        }
    }

    // If there was none and we got here, create a new one.
    let window = match tauri::WebviewWindowBuilder::new(
        &app_handle,
        "bugReport",
        tauri::WebviewUrl::App("bugreporter.html".into()),
    )
    .inner_size(600.0, 500.0)
    .center()
    .title(format!("[{kind}] Report"))
    .focused(true)
    .menu(build_menus(MenuKind::BugReport, &app_handle))
    .initialization_script(
        format!(
            r#"
            if (window.location.origin === 'https://tauri.app') {{
                console.log("hello world from js init script");
                window.__KIND__  = {kind}
            }}
        "#
        )
        .as_str(),
    )
    .build()
    {
        Ok(win) => win,
        Err(err) => {
            eprintln!("Could not create BugReporter window: {:?}", err);
            return Ok(ApplicationError::TauriWindowCreationError);
        }
    };
    match window.show() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Could not show BugReporter window: {:?}", err);
            return Ok(ApplicationError::TauriWindowShowError);
        }
    }

    return Ok(ApplicationError::NoError);
}

#[tauri::command]
async fn send_mail_from_frontend(
    name: Option<String>,
    mail: Option<String>,
    subject: String,
    message: String,
    sendlogs: bool,
    kind: String,
) -> Result<ApplicationError, ()> {
    // First, we need to build a HTML element from the message. Replace all newline chars with <br />
    let message_formatted = message.replace("\n", "<br />");

    // Create a placeholder string for the final message
    let final_message: String;

    if name.is_some() && mail.is_some() {
        final_message = format!(
            "<p>---------<br /><b>{}</b><br /><i>{}</i><br /><br />{message_formatted}</p>",
            name.unwrap(),
            mail.unwrap()
        );
    } else if name.is_some() && mail.is_none() {
        final_message = format!(
            "<p>---------<br /><b>{}</b><br /><br />{message_formatted}</p>",
            name.unwrap()
        );
    } else if name.is_none() && mail.is_some() {
        final_message = format!(
            "<p><---------<br /><i>{}</i><br /><br />{message_formatted}</p>",
            mail.unwrap()
        );
    } else {
        final_message = format!("<p>---------<br />{message_formatted}</p>");
    }

    let mail_type: MessageKind;
    if kind.as_str().contains("BUG") {
        mail_type = MessageKind::Bug(subject);
    } else if kind.as_str().contains("FEEDBACK") {
        mail_type = MessageKind::Feedback(subject);
    } else if kind.as_str().contains("SUPPORT") {
        mail_type = MessageKind::Support(subject);
    } else {
        mail_type = MessageKind::Unknown;
    }

    return Ok(send_mail(mail_type, final_message, sendlogs).await);
}

// This is only used in the panic hook, therefore we don't care about return values
// or anything related. We will abort() anyways.
#[tokio::main]
async fn send_mail_from_panic(kind: MessageKind, body: String) {
    send_mail(kind, body, true).await;
}

#[tauri::command]
fn update_app(requested: bool) {
    // SAFETY: We only use atomics here, so this is fine.
    unsafe {
        if requested {
            UPDATE_REQUESTED.store(1, Ordering::Relaxed);
        } else {
            UPDATE_REQUESTED.store(-1, Ordering::Relaxed);
        }
    }
}

// MARK: Main Function
/// Main application entry function.
fn main() {
    // Set panic hook to send mail if possible
    std::panic::set_hook(Box::new(|info: &PanicHookInfo| {
        // Execute the regular hook
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<dyn Any>",
            },
        };
        let thread = thread::current();
        let name = thread.name().unwrap_or("<unnamed>");

        eprintln!("Thread '{name}' panicked at {location}:\n{msg}");

        // Now hope that everything is flushed!
        // We can then try to send an email with all the data
        let error_heading = format!("Thread {name} panicked at {location}.");
        let error_description = format!("Thread '{name}' panicked at {location}:<br />{msg}");
        let error = MessageKind::Panic((error_heading, error_description));
        let handle = std::thread::spawn(move || {
            if let MessageKind::Panic((_heading, desc)) = error.clone() {
                send_mail_from_panic(error.clone(), format!("<p>{}</p>", desc));
            } else {
                send_mail_from_panic(error.clone(), "<p></p>".to_string());
            }
        });
        match handle.join() {
            Ok(()) => {
                abort();
            }
            Err(err) => {
                eprintln!("Failed to join PanicMailThread: {:?}", err);
                abort();
            }
        }
    }));

    // Rebase all StdOut and StdErr happenings
    #[cfg(not(debug_assertions))]
    match log::activateLogging() {
        Ok(()) => {}
        Err(err) => {
            panic!(
                "Could not redirect StdOut and StdErr successfully: {:?}",
                err
            );
        }
    }

    // Pack files at compile time and write them to disk at runtime... Currently the only way to embed files within the binary cross-platform
    #[cfg(not(target_os = "windows"))]
    let template_file_binary = include_bytes!(r"../../res/Vorlage_Einsatzplan_Leer.docx");
    #[cfg(not(target_os = "windows"))]
    let table_file_binary = include_bytes!(r"../../res/Tabelle_Vorlage_Leer.docx");
    #[cfg(target_os = "windows")]
    let template_file_binary = include_bytes!(r"..\..\res\Vorlage_Einsatzplan_Leer.docx");
    #[cfg(target_os = "windows")]
    let table_file_binary = include_bytes!(r"..\..\res\Tabelle_Vorlage_Leer.docx");

    match directories::BaseDirs::new() {
        None => {
            panic!("Could not get the Windows Base Dirs. Important files will be missing and we cannot get them from anywhere else, so we exit here.")
        }
        Some(dirs) => {
            let appdata_roaming_dir = dirs.data_dir();
            let application_resources_dir = appdata_roaming_dir
                .join("de.philippremy.dtb-kampfrichtereinsatzplaene")
                .join("Resources");
            // Create the folder if it does not exist!
            match std::fs::create_dir_all(application_resources_dir.clone()) {
                Ok(()) => {}
                Err(err) => {
                    panic!("Could not create the AppData dir: {:?}", err);
                }
            }
            // Copy the stuff to there and we should be good to go.
            // Write file at path!
            match std::fs::write(
                application_resources_dir
                    .clone()
                    .join("Vorlage_Einsatzplan_Leer.docx"),
                template_file_binary,
            ) {
                Ok(()) => {}
                Err(e) => panic!("Could not write the template file: {e}"),
            }
            // Write file at path!
            match std::fs::write(
                application_resources_dir
                    .clone()
                    .join("Tabelle_Vorlage_Leer.docx"),
                table_file_binary,
            ) {
                Ok(()) => {}
                Err(e) => panic!("Could not write the table file: {e}"),
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().target(tauri::utils::platform::target_triple().unwrap()).build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Storage::default())
        .invoke_handler(tauri::generate_handler![update_storage_data, create_wettkampf, sync_wk_data_and_open_editor, get_wk_data_to_frontend, sync_to_backend_and_save, sync_to_backend_and_create_docx, sync_to_backend_and_create_pdf, import_wk_file_and_open_editor, show_item_in_folder, open_bug_reporter, send_mail_from_frontend, update_mainwindow_loading_state, update_app])
        .setup(|app| {
            let handle = app.handle().clone();

            // Set App Menu
            app.set_menu(build_menus(MenuKind::Default, &handle)).unwrap();

            // Build the PDF Window
            let pdf_window = tauri::WebviewWindowBuilder::new(&handle, "pdfWindow", tauri::WebviewUrl::App(PathBuf::from("blank.html")))
                .visible(false)
                .accept_first_mouse(false)
                .build()
                .unwrap();
            pdf_window.with_webview(|webview|{

                let mut lockGuard = PLATFORM_WEBVIEW.blocking_lock();
                lockGuard.inner = Some(webview);

            }).unwrap();

            tauri::async_runtime::spawn(async move {

                // Wait until the Window loaded, otherwise we might be too fast and
                // don't receive the events
                // SAFETY: This is safe, we are only accessing atomics
                unsafe {
                    while !MAINWINDOW_LOADED.load(Ordering::Relaxed) {
                        sleep(std::time::Duration::from_millis(100)).await;
                    }
                }

                match handle.updater().unwrap().check().await {
                    Ok(update_option) => {
                        match update_option {
                            Some(update) => {
                                match handle.emit_to("mainUI", "updateIsAvailable", UpdateAvailablePayload{ body: update.body.clone().unwrap(), date: update.date.unwrap().to_string(), version: update.version.clone() }) {
                                    Ok(()) => {},
                                    Err(err) => { eprintln!("Failed to emit updateIsAvailable to frontend: {:?}", err); }
                                }
        
                                // We have to wait for an answer from the frontend
                                // SAFETY: We only use atomic operations, so this is safe
                                unsafe {
                                    while UPDATE_REQUESTED.load(Ordering::Relaxed) == 0 {
                                        sleep(std::time::Duration::from_millis(100)).await;
                                    }
                                    if UPDATE_REQUESTED.load(Ordering::Relaxed) == -1 {
                                        // Update was cancelled, exit the loop
                                        return;
                                    } else if UPDATE_REQUESTED.load(Ordering::Relaxed) == 1 {
                                        // Reset the progress (preventive)
                                        // SAFETY: We are using atomic operations only, so this is safe
                                        UPDATE_PROGRESS.store(0, Ordering::Relaxed);
        
                                        // Update was requested, start with it
                                        match update.download_and_install(|chunk_length, content_length| {
                                            // Get a temp value that we can increase
                                            // SAFETY: We are using atomic operations only, so this is safe
                                            let mut temp_progress;
                                            temp_progress = UPDATE_PROGRESS.load(Ordering::Relaxed);
                                            temp_progress += chunk_length as u64;
                                            UPDATE_PROGRESS.store(temp_progress, Ordering::Relaxed);
                                            match handle.emit_to("mainUI", "updateHasProgress", UpdateProgressPayload{ chunk_len: temp_progress as usize, content_len: content_length }) {
                                                Ok(()) => {},
                                                Err(err) => { eprintln!("Failed to emit updateHasProgress to frontend: {:?}", err); }
                                            }
                                        }, || {
                                            match handle.emit_to("mainUI", "updateIsDownloaded", {}) {
                                                Ok(()) => {},
                                                Err(err) => { eprintln!("Failed to emit updateIsDownloaded to frontend: {:?}", err); }
                                            }
                                        }).await {
                                            Ok(()) => {},
                                            Err(err) => {
                                                eprintln!("Could not download and install the update: {:?}", err);
                                            }
                                        }
                                    }
                                    else {
                                        eprintln!("Unexpected atomic value of UPDATE_REQUESTED found: {:?}", UPDATE_REQUESTED.load(Ordering::Relaxed));
                                    }
                                }
                            }
                            None => {

                            }
                        }
                    },
                    Err(err) => {
                        eprintln!("Could not fetch Update information: {:?}", err);
                        match handle.clone().emit_to("mainUI", "updateThrewError", err.to_string()) {
                            Ok(()) => {},
                            Err(err) => { eprintln!("Failed to emit updateThrewError to frontend: {:?}", err); }
                        }
                    }
                }
            });


            // Handle Menu Events
            app.on_menu_event(|app_handle, ev|{
                match ev.id.0.as_str() {
                    "showLicenses" => {
                        if app_handle.windows().contains_key("licenseWindow") {
                            let windows = app_handle.windows();
                            let license_window = windows.get("licenseWindow").unwrap();
                            license_window.show().unwrap();
                            license_window.set_focus().unwrap();
                            return;
                        }
                        let license_window = tauri::WebviewWindowBuilder::new(&app_handle.clone(), "licenseWindow", tauri::WebviewUrl::App(PathBuf::from("licenses.html")))
                            .inner_size(550.0, 600.0)
                            .title("Open Source Lizenzen".to_string())
                            .center()
                            .focused(true)
                            .menu(build_menus(MenuKind::LicenseWindow, &app_handle))
                            .visible(true)
                            .build()
                            .unwrap();
                        license_window.show().unwrap();
                    },
                    "showLogs" => {
                        unsafe {
                            if STDOUT_FILE.clone().is_some() {
                                match show_item_in_folder(STDOUT_FILE.clone().unwrap()) {
                                    Ok(()) => {}
                                    Err(err) => { eprintln!("Unable to show the logs in Explorer/Finder: {err}") },
                                }
                            } else {
                                eprintln!("STDOUT_FILE and STDERR_FILE are 'None': Either we are running a development build or the piping process was not successful.");
                            }
                        }
                    },
                    "contactSupport" => {
                        open_bug_reporter(String::from("SUPPORT"), app_handle.clone()).unwrap();
                    },
                    "bugReport" => {
                        open_bug_reporter(String::from("BUG"), app_handle.clone()).unwrap();
                    },
                    "feedback" => {
                        open_bug_reporter(String::from("FEEDBACK"), app_handle.clone()).unwrap();
                    },
                    &_ => { println!("The following menu is currently not implemented: {}", ev.id.0); }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .unwrap()
        .run(|app_handle, ev| {
            match ev {
                tauri::RunEvent::WindowEvent { label, event , .. } => {
                    match event {
                        tauri::WindowEvent::CloseRequested { .. } => {
                            if label.as_str() != "pdfWindow" && app_handle.windows().len() < 3 {
                                app_handle.exit(0);
                            }
                        },
                        tauri::WindowEvent::Destroyed => {
                            if label.as_str() != "pdfWindow" && app_handle.windows().len() == 1 {
                                app_handle.exit(0);
                            }
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        });

}

enum MenuKind {
    Default,
    CreateWettkampf,
    Editor,
    BugReport,
    LicenseWindow,
}

fn build_menus(menuKind: MenuKind, app_handle: &AppHandle) -> Menu<Wry> {

    match menuKind {
        MenuKind::Default => {
            let app_submenu = SubmenuBuilder::new(app_handle, "DTB Kampfrichtereinsatzpläne")
                .about(Some(AboutMetadataBuilder::new()
                    .authors(Some(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()]))
                    .license(Some("GPL-3.0-only"))
                    .copyright(Some("© Philipp Remy 2024"))
                    .comments(Some("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB"))
                    .version(Some(APP_VERSION))
                    .website(Some("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene"))
                    .website_label(Some("GitHub Repository"))
                    .build()
                ))
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()
                .unwrap();
            let edit_submenu = SubmenuBuilder::new(app_handle, "Bearbeiten")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()
                .unwrap();
            let view_submenu = SubmenuBuilder::new(app_handle, "Darstellung")
                .fullscreen()
                .build()
                .unwrap();
            let window_submenu = SubmenuBuilder::new(app_handle, "Fenster")
                .minimize()
                .maximize()
                .separator()
                .close_window()
                .build()
                .unwrap();
            let help_submenu = SubmenuBuilder::new(app_handle, "Hilfe")
                .item(&MenuItemBuilder::new("Hilfe").id("help").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Was ist neu?").id("whatsnew").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Support kontaktieren").id("contactSupport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Bug melden").id("bugReport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Feedback geben").id("feedback").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Logs anzeigen").id("showLogs").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Open-Source Lizenzen").id("showLicenses").build(app_handle).unwrap())
                .build()
                .unwrap();
            let menu = MenuBuilder::new(app_handle)
                .build()
                .unwrap();
            menu.append(&app_submenu).unwrap();
            menu.append(&edit_submenu).unwrap();
            menu.append(&view_submenu).unwrap();
            menu.append(&window_submenu).unwrap();
            menu.append(&help_submenu).unwrap();
            menu.set_as_app_menu().unwrap();
            return menu;
        },
        MenuKind::CreateWettkampf => {
            let app_submenu = SubmenuBuilder::new(app_handle, "DTB Kampfrichtereinsatzpläne")
                .about(Some(AboutMetadataBuilder::new()
                    .authors(Some(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()]))
                    .license(Some("GPL-3.0-only"))
                    .copyright(Some("© Philipp Remy 2024"))
                    .comments(Some("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB"))
                    .version(Some(APP_VERSION))
                    .website(Some("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene"))
                    .website_label(Some("GitHub Repository"))
                    .build()
                ))
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()
                .unwrap();
            let edit_submenu = SubmenuBuilder::new(app_handle, "Bearbeiten")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()
                .unwrap();
            let view_submenu = SubmenuBuilder::new(app_handle, "Darstellung")
                .fullscreen()
                .build()
                .unwrap();
            let window_submenu = SubmenuBuilder::new(app_handle, "Fenster")
                .minimize()
                .maximize()
                .separator()
                .close_window()
                .build()
                .unwrap();
            let help_submenu = SubmenuBuilder::new(app_handle, "Hilfe")
                .item(&MenuItemBuilder::new("Hilfe").id("help").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Was ist neu?").id("whatsnew").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Support kontaktieren").id("contactSupport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Bug melden").id("bugReport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Feedback geben").id("feedback").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Logs anzeigen").id("showLogs").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Open-Source Lizenzen").id("showLicenses").build(app_handle).unwrap())
                .build()
                .unwrap();
            let menu = MenuBuilder::new(app_handle)
                .build()
                .unwrap();
            menu.append(&app_submenu).unwrap();
            menu.append(&edit_submenu).unwrap();
            menu.append(&view_submenu).unwrap();
            menu.append(&window_submenu).unwrap();
            menu.append(&help_submenu).unwrap();
            menu.set_as_app_menu().unwrap();
            return menu;
        },
        MenuKind::Editor => {
            let app_submenu = SubmenuBuilder::new(app_handle, "DTB Kampfrichtereinsatzpläne")
                .about(Some(AboutMetadataBuilder::new()
                    .authors(Some(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()]))
                    .license(Some("GPL-3.0-only"))
                    .copyright(Some("© Philipp Remy 2024"))
                    .comments(Some("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB"))
                    .version(Some(APP_VERSION))
                    .website(Some("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene"))
                    .website_label(Some("GitHub Repository"))
                    .build()
                ))
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()
                .unwrap();
            let edit_submenu = SubmenuBuilder::new(app_handle, "Bearbeiten")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()
                .unwrap();
            let view_submenu = SubmenuBuilder::new(app_handle, "Darstellung")
                .fullscreen()
                .build()
                .unwrap();
            let window_submenu = SubmenuBuilder::new(app_handle, "Fenster")
                .minimize()
                .maximize()
                .separator()
                .close_window()
                .build()
                .unwrap();
            let help_submenu = SubmenuBuilder::new(app_handle, "Hilfe")
                .item(&MenuItemBuilder::new("Hilfe").id("help").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Was ist neu?").id("whatsnew").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Support kontaktieren").id("contactSupport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Bug melden").id("bugReport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Feedback geben").id("feedback").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Logs anzeigen").id("showLogs").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Open-Source Lizenzen").id("showLicenses").build(app_handle).unwrap())
                .build()
                .unwrap();
            let menu = MenuBuilder::new(app_handle)
                .build()
                .unwrap();
            menu.append(&app_submenu).unwrap();
            menu.append(&edit_submenu).unwrap();
            menu.append(&view_submenu).unwrap();
            menu.append(&window_submenu).unwrap();
            menu.append(&help_submenu).unwrap();
            menu.set_as_app_menu().unwrap();
            return menu;
        },
        MenuKind::BugReport => {
            let app_submenu = SubmenuBuilder::new(app_handle, "DTB Kampfrichtereinsatzpläne")
                .about(Some(AboutMetadataBuilder::new()
                    .authors(Some(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()]))
                    .license(Some("GPL-3.0-only"))
                    .copyright(Some("© Philipp Remy 2024"))
                    .comments(Some("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB"))
                    .version(Some(APP_VERSION))
                    .website(Some("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene"))
                    .website_label(Some("GitHub Repository"))
                    .build()
                ))
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()
                .unwrap();
            let edit_submenu = SubmenuBuilder::new(app_handle, "Bearbeiten")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()
                .unwrap();
            let view_submenu = SubmenuBuilder::new(app_handle, "Darstellung")
                .fullscreen()
                .build()
                .unwrap();
            let window_submenu = SubmenuBuilder::new(app_handle, "Fenster")
                .minimize()
                .maximize()
                .separator()
                .close_window()
                .build()
                .unwrap();
            let help_submenu = SubmenuBuilder::new(app_handle, "Hilfe")
                .item(&MenuItemBuilder::new("Hilfe").id("help").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Was ist neu?").id("whatsnew").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Support kontaktieren").id("contactSupport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Bug melden").id("bugReport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Feedback geben").id("feedback").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Logs anzeigen").id("showLogs").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Open-Source Lizenzen").id("showLicenses").build(app_handle).unwrap())
                .build()
                .unwrap();
            let menu = MenuBuilder::new(app_handle)
                .build()
                .unwrap();
            menu.append(&app_submenu).unwrap();
            menu.append(&edit_submenu).unwrap();
            menu.append(&view_submenu).unwrap();
            menu.append(&window_submenu).unwrap();
            menu.append(&help_submenu).unwrap();
            menu.set_as_app_menu().unwrap();
            return menu;
        },
        MenuKind::LicenseWindow => {
            let app_submenu = SubmenuBuilder::new(app_handle, "DTB Kampfrichtereinsatzpläne")
                .about(Some(AboutMetadataBuilder::new()
                    .authors(Some(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()]))
                    .license(Some("GPL-3.0-only"))
                    .copyright(Some("© Philipp Remy 2024"))
                    .comments(Some("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB"))
                    .version(Some(APP_VERSION))
                    .website(Some("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene"))
                    .website_label(Some("GitHub Repository"))
                    .build()
                ))
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()
                .unwrap();
            let edit_submenu = SubmenuBuilder::new(app_handle, "Bearbeiten")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()
                .unwrap();
            let view_submenu = SubmenuBuilder::new(app_handle, "Darstellung")
                .fullscreen()
                .build()
                .unwrap();
            let window_submenu = SubmenuBuilder::new(app_handle, "Fenster")
                .minimize()
                .maximize()
                .separator()
                .close_window()
                .build()
                .unwrap();
            let help_submenu = SubmenuBuilder::new(app_handle, "Hilfe")
                .item(&MenuItemBuilder::new("Hilfe").id("help").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Was ist neu?").id("whatsnew").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Support kontaktieren").id("contactSupport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Bug melden").id("bugReport").build(app_handle).unwrap())
                .item(&MenuItemBuilder::new("Feedback geben").id("feedback").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Logs anzeigen").id("showLogs").build(app_handle).unwrap())
                .separator()
                .item(&MenuItemBuilder::new("Open-Source Lizenzen").id("showLicenses").build(app_handle).unwrap())
                .build()
                .unwrap();
            let menu = MenuBuilder::new(app_handle)
                .build()
                .unwrap();
            menu.append(&app_submenu).unwrap();
            menu.append(&edit_submenu).unwrap();
            menu.append(&view_submenu).unwrap();
            menu.append(&window_submenu).unwrap();
            menu.append(&help_submenu).unwrap();
            menu.set_as_app_menu().unwrap();
            return menu;
        },
    };

}