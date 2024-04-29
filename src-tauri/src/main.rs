// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_doc_comments)]
#![allow(non_snake_case)]

use std::path::PathBuf;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use tauri::{AppHandle, Menu, MenuItem, State};
use crate::FFI::create_tables_docx;
use crate::types::{ApplicationError, FrontendStorage, Storage};

/// Declares the usage of crate-wide modules.
mod types;
mod FFI;
mod log;

// Statics
static VERSION: &str = env!("CARGO_PKG_VERSION");

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

    // Create all the Menus
    let mut window_menu = tauri::Menu::new();
    let about_menu: tauri::MenuItem = tauri::MenuItem::About("DTB Kampfrichtereinsatzpläne".to_string(), tauri::AboutMetadata::new()
        .authors(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()])
        .license("GPL-3.0-only".to_string())
        .copyright("© Philipp Remy 2024".to_string())
        .comments("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB".to_string())
        .version(VERSION.to_string())
        .website("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene".to_string())
        .website_label("GitHub Repository".to_string())
    );
    let help_menu_item = tauri::CustomMenuItem::new("help", "Hilfe");
    let whats_new_item = tauri::CustomMenuItem::new("whatsnew", "Was ist neu?");
    let contact_support_item = tauri::CustomMenuItem::new("contactSupport", "Support kontaktieren");
    let bug_report_item = tauri::CustomMenuItem::new("bugReport", "Bug melden");
    let feedback_item = tauri::CustomMenuItem::new("feedback", "Feedback geben");
    let log_item = tauri::CustomMenuItem::new("showLogs", "Logs anzeigen");
    let licenses_item = tauri::CustomMenuItem::new("showLicenses", "Open Source Lizenzen anzeigen");
    let other_submenu = tauri::Submenu::new("Sonstiges", Menu::new()
        .add_item(help_menu_item)
        .add_item(whats_new_item)
        .add_native_item(MenuItem::Separator)
        .add_item(contact_support_item)
        .add_item(bug_report_item)
        .add_item(feedback_item)
        .add_native_item(MenuItem::Separator)
        .add_item(log_item)
        .add_native_item(MenuItem::Separator)
        .add_item(licenses_item)
    );
    let application_submenu = tauri::Submenu::new("DTB Kampfrichtereinsatzpläne".to_string(), Menu::new()
        .add_native_item(about_menu)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Services)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Hide)
        .add_native_item(MenuItem::Minimize)
        .add_native_item(MenuItem::EnterFullScreen)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::CloseWindow)
        .add_native_item(MenuItem::Quit)
    );

    // Build them
    window_menu = window_menu.add_submenu(application_submenu);
    window_menu = window_menu.add_submenu(other_submenu);

    let create_wettkampf_window = match tauri::WindowBuilder::new(&app_handle, "createWettkampf", tauri::WindowUrl::App(PathBuf::from("createWettkampf.html")))
        .inner_size(515.0, 600.0)
        .title("Wettkampf erstellen")
        .focused(true)
        .menu(window_menu)
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
/// Returns: A Result always containing an Ok(ApplicationError) value - never void or something went terribly wrong.
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

    // Create all the Menus
    let mut window_menu = tauri::Menu::new();
    let about_menu: tauri::MenuItem = tauri::MenuItem::About("DTB Kampfrichtereinsatzpläne".to_string(), tauri::AboutMetadata::new()
        .authors(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()])
        .license("GPL-3.0-only".to_string())
        .copyright("© Philipp Remy 2024".to_string())
        .comments("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB".to_string())
        .version(VERSION.to_string())
        .website("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene".to_string())
        .website_label("GitHub Repository".to_string())
    );
    let save_wk_menu = tauri::CustomMenuItem::new("saveWk", "Wettkampf speichern...");
    let save_wk_under_menu = tauri::CustomMenuItem::new("saveWkUnder", "Wettkampf speichern unter...");
    let create_docx_menu = tauri::CustomMenuItem::new("createDocx", "Pläne als .docx speichern...");
    let create_pdf_menu = tauri::CustomMenuItem::new("createPdf", "Pläne als .pdf speichern...");
    let file_submenu = tauri::Submenu::new("Aktionen", Menu::new()
        .add_item(save_wk_menu)
        .add_item(save_wk_under_menu)
        .add_native_item(MenuItem::Separator)
        .add_item(create_docx_menu)
        .add_item(create_pdf_menu)
    );
    let help_menu_item = tauri::CustomMenuItem::new("help", "Hilfe");
    let whats_new_item = tauri::CustomMenuItem::new("whatsnew", "Was ist neu?");
    let contact_support_item = tauri::CustomMenuItem::new("contactSupport", "Support kontaktieren");
    let bug_report_item = tauri::CustomMenuItem::new("bugReport", "Bug melden");
    let feedback_item = tauri::CustomMenuItem::new("feedback", "Feedback geben");
    let log_item = tauri::CustomMenuItem::new("showLogs", "Logs anzeigen");
    let licenses_item = tauri::CustomMenuItem::new("showLicenses", "Open Source Lizenzen anzeigen");
    let other_submenu = tauri::Submenu::new("Sonstiges", Menu::new()
        .add_item(help_menu_item)
        .add_item(whats_new_item)
        .add_native_item(MenuItem::Separator)
        .add_item(contact_support_item)
        .add_item(bug_report_item)
        .add_item(feedback_item)
        .add_native_item(MenuItem::Separator)
        .add_item(log_item)
        .add_native_item(MenuItem::Separator)
        .add_item(licenses_item)
    );
    let application_submenu = tauri::Submenu::new("DTB Kampfrichtereinsatzpläne".to_string(), Menu::new()
        .add_native_item(about_menu)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Services)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Hide)
        .add_native_item(MenuItem::Minimize)
        .add_native_item(MenuItem::EnterFullScreen)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::CloseWindow)
        .add_native_item(MenuItem::Quit)
    );

    // Build them
    window_menu = window_menu.add_submenu(application_submenu);
    window_menu = window_menu.add_submenu(file_submenu);
    window_menu = window_menu.add_submenu(other_submenu);

    // Create the Editor Window
    let editor_window = match tauri::WindowBuilder::new(&app_handle, "editor", tauri::WindowUrl::App(PathBuf::from("editor.html")))
    .inner_size(1250.0, 800.0)
    .title(format!["{} (nicht gespeichert)", data.wk_name])
    .focused(true)
    .center()
    .menu(window_menu)
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
                frontend_storage.wk_replacement_judges = Some(Vec::new());
            } else {
                frontend_storage.wk_replacement_judges = Some((*guard).clone());
            }
        }
        Err(_err) => return Err(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgingtables.lock() {
        Ok(guard) => {
            if guard.is_empty() {
                frontend_storage.wk_judgingtables = Some(HashMap::new());
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

/// Function to sync all stuff and create the plans using FFI
#[tauri::command]
async fn sync_to_backend_and_create_docx(frontendstorage: FrontendStorage, filepath: String, storage: State<'_, Storage>) -> Result<ApplicationError, ()> {

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
                Some(map) => { map },
                None => { HashMap::new() },
            };
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    return Ok(create_tables_docx(storage.inner(), PathBuf::from(filepath)).unwrap());

}

#[tauri::command]
async fn sync_to_backend_and_create_pdf(frontendstorage: FrontendStorage, _filepath: String, storage: State<'_, Storage>) -> Result<ApplicationError, ()> {

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
                Some(map) => { map },
                None => { HashMap::new() },
            };
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    return Ok(ApplicationError::NoError);

}

// Function for loading a file from disk and importing this into frontend storage
// Then open the editor
#[tauri::command]
async fn import_wk_file_and_open_editor(filepath: String, storage: State<'_, Storage>, app_handle: AppHandle) -> Result <ApplicationError, ()> {

    // Deserialize the file
    let imported_storage: Storage = match serde_json::from_reader(File::open(filepath).unwrap()) {
        Ok(storage) => {storage},
        Err(_err) => {return Ok(ApplicationError::JSONDeserializeImporterError)},
    };

    // Update the storage
    match storage.wk_name.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_name.lock().unwrap().clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_place.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_place.lock().unwrap().clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_date.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_date.lock().unwrap().clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgesmeeting_time.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_judgesmeeting_time.lock().unwrap().clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_responsible_person.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_responsible_person.lock().unwrap().clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_replacement_judges.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_replacement_judges.lock().unwrap().clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    match storage.wk_judgingtables.lock() {
        Ok(mut guard) => {
            *guard = imported_storage.wk_judgingtables.lock().unwrap().clone();
            drop(guard);
        },
        Err(_err) => return Ok(ApplicationError::MutexPoisonedError),
    }

    // Create all the Menus
    let mut window_menu = tauri::Menu::new();
    let about_menu: tauri::MenuItem = tauri::MenuItem::About("DTB Kampfrichtereinsatzpläne".to_string(), tauri::AboutMetadata::new()
        .authors(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()])
        .license("GPL-3.0-only".to_string())
        .copyright("© Philipp Remy 2024".to_string())
        .comments("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB".to_string())
        .version(VERSION.to_string())
        .website("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene".to_string())
        .website_label("GitHub Repository".to_string())
    );
    let save_wk_menu = tauri::CustomMenuItem::new("saveWk", "Wettkampf speichern...");
    let save_wk_under_menu = tauri::CustomMenuItem::new("saveWkUnder", "Wettkampf speichern unter...");
    let create_docx_menu = tauri::CustomMenuItem::new("createDocx", "Pläne als .docx speichern...");
    let create_pdf_menu = tauri::CustomMenuItem::new("createPdf", "Pläne als .pdf speichern...");
    let file_submenu = tauri::Submenu::new("Aktionen", Menu::new()
        .add_item(save_wk_menu)
        .add_item(save_wk_under_menu)
        .add_native_item(MenuItem::Separator)
        .add_item(create_docx_menu)
        .add_item(create_pdf_menu)
    );
    let help_menu_item = tauri::CustomMenuItem::new("help", "Hilfe");
    let whats_new_item = tauri::CustomMenuItem::new("whatsnew", "Was ist neu?");
    let contact_support_item = tauri::CustomMenuItem::new("contactSupport", "Support kontaktieren");
    let bug_report_item = tauri::CustomMenuItem::new("bugReport", "Bug melden");
    let feedback_item = tauri::CustomMenuItem::new("feedback", "Feedback geben");
    let log_item = tauri::CustomMenuItem::new("showLogs", "Logs anzeigen");
    let licenses_item = tauri::CustomMenuItem::new("showLicenses", "Open Source Lizenzen anzeigen");
    let other_submenu = tauri::Submenu::new("Sonstiges", Menu::new()
        .add_item(help_menu_item)
        .add_item(whats_new_item)
        .add_native_item(MenuItem::Separator)
        .add_item(contact_support_item)
        .add_item(bug_report_item)
        .add_item(feedback_item)
        .add_native_item(MenuItem::Separator)
        .add_item(log_item)
        .add_native_item(MenuItem::Separator)
        .add_item(licenses_item)
    );
    let application_submenu = tauri::Submenu::new("DTB Kampfrichtereinsatzpläne".to_string(), Menu::new()
        .add_native_item(about_menu)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Services)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Hide)
        .add_native_item(MenuItem::Minimize)
        .add_native_item(MenuItem::EnterFullScreen)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::CloseWindow)
        .add_native_item(MenuItem::Quit)
    );

    // Build them
    window_menu = window_menu.add_submenu(application_submenu);
    window_menu = window_menu.add_submenu(file_submenu);
    window_menu = window_menu.add_submenu(other_submenu);

    // Open the Editor!
    let editor_window = match tauri::WindowBuilder::new(&app_handle, "editor", tauri::WindowUrl::App(PathBuf::from("editor.html")))
        .inner_size(1250.0, 800.0)
        .title(format!["{} (gespeichert)", imported_storage.wk_name.lock().unwrap()])
        .focused(true)
        .center()
        .menu(window_menu)
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

// MARK: Main Function
/// Main application entry function.
fn main() {

    // Rebase all StdOut and StdErr happenings
    match log::activateLogging() {
        Ok(()) => {}
        Err(err) => {
            panic!("Could not redirect StdOut and StdErr successfully: {:?}", err);
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
        None => {panic!("Could not get the Windows Base Dirs. Important files will be missing and we cannot get them from anywhere else, so we exit here.")}
        Some(dirs) => {
            let appdata_roaming_dir = dirs.data_dir();
            let application_resources_dir = appdata_roaming_dir.join("de.philippremy.dtb-kampfrichtereinsatzplaene").join("Resources");
            // Create the folder if it does not exist!
            match std::fs::create_dir_all(application_resources_dir.clone()) {
                Ok(()) => {}
                Err(err) => {
                    panic!("Could not create the AppData dir: {:?}", err);
                }
            }
            // Copy the stuff to there and we should be good to go.
            // Write file at path!
            match std::fs::write(application_resources_dir.clone().join("Vorlage_Einsatzplan_Leer.docx"), template_file_binary) {
                Ok(()) => {}
                Err(e) => panic!("Could not write the template file: {e}"),
            }
            // Write file at path!
            match std::fs::write(application_resources_dir.clone().join("Tabelle_Vorlage_Leer.docx"), table_file_binary) {
                Ok(()) => {},
                Err(e) => panic!("Could not write the table file: {e}"),
            }
        }
    }

    // Create all the Menus
    let mut window_menu = tauri::Menu::new();
    let about_menu: tauri::MenuItem = tauri::MenuItem::About("DTB Kampfrichtereinsatzpläne".to_string(), tauri::AboutMetadata::new()
        .authors(vec!["Philipp Remy <philipp.remy@dtb.de>".to_string()])
        .license("GPL-3.0-only".to_string())
        .copyright("© Philipp Remy 2024".to_string())
        .comments("Ein Programm zum Erstellen von Kampfrichtereinsatzplänen bei Rhönradwettkämpfen im DTB".to_string())
        .version(VERSION.to_string())
        .website("https://github.com/philippremy/dtb-kampfrichtereinsatzplaene".to_string())
        .website_label("GitHub Repository".to_string())
    );
    let open_wk_menu = tauri::CustomMenuItem::new("openWk", "Wettkampf öffnen");
    let create_wk_menu = tauri::CustomMenuItem::new("createWk", "Wettkampf erstellen");
    let file_submenu = tauri::Submenu::new("Aktionen", Menu::new()
        .add_item(create_wk_menu)
        .add_item(open_wk_menu)
    );
    let help_menu_item = tauri::CustomMenuItem::new("help", "Hilfe");
    let whats_new_item = tauri::CustomMenuItem::new("whatsnew", "Was ist neu?");
    let contact_support_item = tauri::CustomMenuItem::new("contactSupport", "Support kontaktieren");
    let bug_report_item = tauri::CustomMenuItem::new("bugReport", "Bug melden");
    let feedback_item = tauri::CustomMenuItem::new("feedback", "Feedback geben");
    let log_item = tauri::CustomMenuItem::new("showLogs", "Logs anzeigen");
    let licenses_item = tauri::CustomMenuItem::new("showLicenses", "Open Source Lizenzen anzeigen");
    let other_submenu = tauri::Submenu::new("Sonstiges", Menu::new()
        .add_item(help_menu_item)
        .add_item(whats_new_item)
        .add_native_item(MenuItem::Separator)
        .add_item(contact_support_item)
        .add_item(bug_report_item)
        .add_item(feedback_item)
        .add_native_item(MenuItem::Separator)
        .add_item(log_item)
        .add_native_item(MenuItem::Separator)
        .add_item(licenses_item)
    );
    let application_submenu = tauri::Submenu::new("DTB Kampfrichtereinsatzpläne".to_string(), Menu::new()
        .add_native_item(about_menu)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Services)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Hide)
        .add_native_item(MenuItem::Minimize)
        .add_native_item(MenuItem::EnterFullScreen)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::CloseWindow)
        .add_native_item(MenuItem::Quit)
    );

    // Build them
    window_menu = window_menu.add_submenu(application_submenu);
    window_menu = window_menu.add_submenu(file_submenu);
    window_menu = window_menu.add_submenu(other_submenu);

    tauri::Builder::default()
        .menu(window_menu)
        .manage(Storage::default())
        .invoke_handler(tauri::generate_handler![update_storage_data, create_wettkampf, sync_wk_data_and_open_editor, get_wk_data_to_frontend, sync_to_backend_and_save, sync_to_backend_and_create_docx, sync_to_backend_and_create_pdf, import_wk_file_and_open_editor])
        .setup(|_app| {
            Ok(())
        })
        .build(tauri::generate_context!())
        .unwrap()
        .run(|_app_handle, _ev| {

        });
}
