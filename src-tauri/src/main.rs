// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_doc_comments)]
#![allow(non_snake_case)]

use std::path::PathBuf;
use std::process::exit;
use std::sync::Mutex;
use std::collections::HashMap;
use tauri::{AppHandle, State};
use crate::FFI::create_tables_docx;
use crate::types::{ApplicationError, FrontendStorage, Kampfgericht, Kampfrichter, Storage};

/// Declares the usage of crate-wide modules.
mod types;
mod FFI;

// MARK: Func: Update Storage Data
/// Function to update the global storage from the frontend.
/// Param 1: The frontend storage struct provided by Javascript
/// Param 2: The managed Storage state object provided by Tauri.
/// Returns: An ApplicationError to be handled by the frontend.
#[tauri::command]
fn update_storage_data(frontend_storage: FrontendStorage, storage: State<Storage>) -> ApplicationError {

    // Update all data. First, lock the relevant object and then update it.
    // Immediately after drop the lock.

    // For wk_name
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_name;
            drop(guard);
        }
        Err(_err) => {
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_date
    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_date;
            drop(guard);
        }
        Err(_err) => {
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_responsible_person
    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_responsible_person;
            drop(guard);
        }
        Err(_err) => {
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_judgesmeeting_time
    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_judgesmeeting_time;
            drop(guard);
        }
        Err(_err) => {
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_replacement_judges
    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_replacement_judges.unwrap();
            drop(guard);
        }
        Err(_err) => {
            return ApplicationError::MutexPoisonedError;
        }
    };

    // For wk_judgingtables
    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = frontend_storage.wk_judgingtables.unwrap();
            drop(guard);
        }
        Err(_err) => {
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
    let create_wettkampf_window = match tauri::WindowBuilder::new(&app_handle, "createWettkampf", tauri::WindowUrl::App(PathBuf::from("createWettkampf.html")))
        .inner_size(515.0, 600.0)
        .title("Wettkampf erstellen")
        .focused(true)
        .center()
        .build()
    {
        Ok(window) => {window},
        Err(_err) => return ApplicationError::TauriWindowCreationError,
    };
    match create_wettkampf_window.show() {
        Ok(()) => {},
        Err(_err) => return ApplicationError::TauriWindowShowError,
    }
    return ApplicationError::NoError;
}

// MARK: Func: Sync WK Data and open Editor
/// Syncs the initial WK data and hands off to the GUI WK Editor.
/// Param 1: FrontendStorage data struct (provided by the Frontend) through serde::Deserialize
/// Param 2: State<'_ Storage> global storage provided by Tauri.
/// Returns: A Result always containing a Ok(ApplicationError) value - never void or something went terribly wrong.
/// CAVEATS: async functions cannot simply use borrowed data like State<T>, so we need the anonymous lifetime specifier "'_" and have to return a Result.
#[tauri::command]
async fn sync_wk_data_and_open_editor(data: FrontendStorage, storage: State<'_, Storage>, app_handle: AppHandle) -> Result<ApplicationError, ()> {

    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = data.wk_name.clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = data.wk_place;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = data.wk_date;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = data.wk_judgesmeeting_time;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = data.wk_responsible_person;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    // Create the Editor Window
    let editor_window = match tauri::WindowBuilder::new(&app_handle, "editor", tauri::WindowUrl::App(PathBuf::from("editor.html")))
    .inner_size(1250.0, 800.0)
    .title(format!["{} (nicht gespeichert)", data.wk_name])
    .focused(true)
    .center()
    .build()
    {
        Ok(window) => {window},
        Err(_err) => return Ok(ApplicationError::TauriWindowCreationError),
    };
    match editor_window.show() {
        Ok(()) => {},
        Err(_err) => return Ok(ApplicationError::TauriWindowShowError),
    }
    
    return Ok(ApplicationError::NoError);
}

#[tauri::command]
async fn get_wk_data_to_frontend(storage: State<'_, Storage>) -> Result<FrontendStorage, ApplicationError> {
    let mut frontend_storage = FrontendStorage::default();
    match storage.wk_name.lock() {
        Ok(guard) => {
            frontend_storage.wk_name = (*guard).clone();
            drop(guard);
        },
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_place.lock() {
        Ok(guard) => {
            frontend_storage.wk_place = (*guard).clone();
            drop(guard);
        },
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_date.lock() {
        Ok(guard) => {
            frontend_storage.wk_date = (*guard).clone();
            drop(guard);
        },
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(guard) => {
            frontend_storage.wk_judgesmeeting_time = (*guard).clone();
            drop(guard);
        },
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_responsible_person.lock() {
        Ok(guard) => {
            frontend_storage.wk_responsible_person = (*guard).clone();
            drop(guard);
        },
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_replacement_judges.lock() {
        Ok(guard) => {
            if guard.is_empty() {
                frontend_storage.wk_replacement_judges = None;
            } else {
                frontend_storage.wk_replacement_judges = Some((*guard).clone());
            }
        }
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgingtables.lock() {
        Ok(guard) => {
            if guard.is_empty() {
                frontend_storage.wk_judgingtables = None;
            } else {
                frontend_storage.wk_judgingtables = Some((*guard).clone());
            }
        }
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    return Ok(frontend_storage);
}

#[tauri::command]
async fn sync_to_backend_and_save(frontendstorage: FrontendStorage, filepath: String, storage: State<'_, Storage>) -> Result<ApplicationError, ()> {

    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_name.clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_place;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_date;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_judgesmeeting_time;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = frontendstorage.wk_responsible_person;
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_replacement_judges {
                Some(map) => {map},
                None => {Vec::new()},
            };
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = match frontendstorage.wk_judgingtables {
                Some(map) => {map},
                None => {HashMap::new()},
            };
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    // Serialize data
    let serialized_data = match serde_json::to_string(storage.inner()) {
        Ok(data) => {data}
        Err(_err) => { return Ok(ApplicationError::JSONSerializeError) }
    };

    // Write file at path!
    match std::fs::write(filepath, serialized_data) {
        Ok(()) => {},
        Err(_err) => { return Ok(ApplicationError::RustWriteFileError) }
    }

    return Ok(ApplicationError::NoError);

}

// MARK: Main Function
/// Main application entry function.
fn main() {
    tauri::Builder::default()
        .manage(Storage::default())
        .invoke_handler(tauri::generate_handler![update_storage_data, create_wettkampf, sync_wk_data_and_open_editor, get_wk_data_to_frontend, sync_to_backend_and_save])
        .setup(|_app| {
            Ok(())
        })
        .build(tauri::generate_context!())
        .unwrap()
        .run(|_app_handle, _ev| {

        });
}
