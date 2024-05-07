// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_doc_comments)]
#![allow(non_snake_case)]

use std::path::PathBuf;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::process::Command;
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::types::{PrintToPdfOptions};
use tauri::{AppHandle, Manager, Menu, MenuItem, State, WindowBuilder};
use crate::ChromeFetcher::{Fetcher, FetcherOptions, Revision};
use crate::FFI::{create_tables_docx, create_tables_pdf};
use crate::types::{ApplicationError, FrontendStorage, Storage};
#[cfg(target_os = "linux")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::sync::Mutex;

/// Declares the usage of crate-wide modules.
mod types;
mod FFI;
mod log;
mod ChromeFetcher;

// Linux struct
#[cfg(target_os = "linux")]
pub struct DbusState(Mutex<Option<dbus::blocking::SyncConnection>>);

// Statics
static VERSION: &str = env!("CARGO_PKG_VERSION");
static mut SAVE_PATH: Option<String> = None;
static mut CHROME_BIN: Option<PathBuf> = None;

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
async fn get_wk_data_to_frontend(storage: State<'_, Storage>) -> Result<(FrontendStorage, Option<String>), ApplicationError> {
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

    unsafe { return Ok((frontend_storage, SAVE_PATH.clone())) };
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
async fn sync_to_backend_and_create_pdf(frontendstorage: FrontendStorage, filepath: String, storage: State<'_, Storage>) -> Result<ApplicationError, ()> {

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

    // Process backend library docx and html
    let library_code = create_tables_pdf(storage.inner(), PathBuf::from(filepath.clone())).unwrap();

    // Check if this succeeded.
    if library_code != ApplicationError::NoError {
        return Ok(library_code);
    }

    // SAFETY: This will only be fetched from different window instances, no race conditions
    // are to be expected. No human is that fast.
    let chromium_binary = unsafe { CHROME_BIN.clone() };

    // If this is none, that is a very bad sign. Maybe a race condition nobody saw in advance :(?
    if chromium_binary.is_none() {
        return Ok(ApplicationError::ChromiumBinaryIsUnexpectedlyNone);
    }

    // Now use our local chromium install to generate the pdf
    let browser = match Browser::new(LaunchOptions::default_builder().headless(true).enable_logging(true).path(chromium_binary).build().unwrap()) {
        Ok(browser) => { browser },
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::BrowserCouldNotBeBuild) }
    };
    let tab = match browser.new_tab() {
        Ok(tab) => { tab },
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::NewTabCouldNotBeCreated) }
    };

    // Fetch the generated html file
    let generated_html = filepath.clone().replace(".pdf", "_temp.html");
    let generated_docx = filepath.clone().replace(".pdf", "_temp.docx");

    // Navigate to it
    let mut generated_html_tab = match tab.navigate_to(&format!["file://{}", generated_html.as_str()]) {
        Ok(tab) => { tab },
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::NavigationToGeneratedHTMLFileFailed) }
    };

    // Wait for navigation to finish
    generated_html_tab = match generated_html_tab.wait_until_navigated() {
        Ok(tab) => { tab },
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::WaitingForNavigationFailed) }
    };

    // Print to pdf and get the binary data
    let mut pdf_options = PrintToPdfOptions::default();
    pdf_options.paper_height = Some(11.7);
    pdf_options.paper_width = Some(8.3);
    pdf_options.display_header_footer = Some(false);
    pdf_options.margin_bottom = Some(0.0);
    pdf_options.margin_left = Some(0.0);
    pdf_options.margin_right = Some(0.0);
    pdf_options.margin_top = Some(0.0);
    pdf_options.scale = Some(1.0);
    let pdf_data = match generated_html_tab.print_to_pdf(Some(pdf_options)) {
        Ok(data) => { data },
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::PDFGenerationInChromiumFailed) }
    };

    // Write the pdf contents to file!
    match std::fs::write(filepath.clone(), pdf_data) {
        Ok(()) => {},
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::WritingPDFDataToDiskFailed) }
    }

    // Delete temporary files
    match std::fs::remove_file(generated_html) {
        Ok(()) => {},
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::RemovalOfTemporaryGeneratedFilesFailed) }
    }

    // Delete temporary files
    match std::fs::remove_file(generated_docx) {
        Ok(()) => {},
        Err(err) => { println!("{:?}", err); return Ok(ApplicationError::RemovalOfTemporaryGeneratedFilesFailed) }
    }

    return Ok(ApplicationError::NoError);
}

// Function for loading a file from disk and importing this into frontend storage
// Then open the editor
#[tauri::command]
async fn import_wk_file_and_open_editor(filepath: String, storage: State<'_, Storage>, app_handle: AppHandle) -> Result <ApplicationError, ()> {

    // Set the static thing so we know where this was saved!
    unsafe { SAVE_PATH = Some(filepath.clone()) };

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

// Function for checking if we have a chrome binary installed on the system
#[tauri::command]
fn check_for_chrome_binary() -> bool {

    match directories::BaseDirs::new() {
        None => { panic!("Could not get the Windows Base Dirs. Important files will be missing and we cannot get them from anywhere else, so we exit here.") }
        Some(dirs) => {
            let appdata_roaming_dir = dirs.data_dir();
            let application_externals_dir = appdata_roaming_dir.join("de.philippremy.dtb-kampfrichtereinsatzplaene").join("Externals");
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
                let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294836".to_string()));
            #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
                let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294832".to_string()));
            #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
                let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294832".to_string()));
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294832".to_string()));
            let fetcher = Fetcher::new(fetcher_options);
            let fetched_instance = match fetcher.fetch() {
                Ok(path) => { path },
                Err(_err) => { return false }
            };
            // SAFETY: This will only be fetched from different window instances, no race conditions
            // are to be expected. No human is that fast.
            unsafe { CHROME_BIN = Some(fetched_instance.clone()) };
        }
    }

    return true;
}

#[tauri::command]
async fn download_chrome() -> Result<ApplicationError, ()> {
    match directories::BaseDirs::new() {
        None => { panic!("Could not get the Windows Base Dirs. Important files will be missing and we cannot get them from anywhere else, so we exit here.") }
        Some(dirs) => {
            let appdata_roaming_dir = dirs.data_dir();
            let application_externals_dir = appdata_roaming_dir.join("de.philippremy.dtb-kampfrichtereinsatzplaene").join("Externals");
	    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294836".to_string()));
	    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
	    let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294832".to_string()));
	    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
	    let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294832".to_string()));
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let fetcher_options = FetcherOptions::new().with_allow_standard_dirs(false).with_allow_download(true).with_install_dir(Some(application_externals_dir)).with_revision(Revision::Specific("1294832".to_string()));
	    let fetcher = Fetcher::new(fetcher_options);
            let fetched_instance = match fetcher.fetch() {
                Ok(path) => { path },
                Err(err) => { println!("{:?}", err); return Ok(ApplicationError::ChromeDownloadError) }
            };
            // SAFETY: This will only be fetched from different window instances, no race conditions
            // are to be expected. No human is that fast.
            unsafe { CHROME_BIN = Some(fetched_instance.clone()) };
        }
    }
    return Ok(ApplicationError::NoError);
}

#[tauri::command]
fn check_if_pdf_is_available() -> bool {
    unsafe {
        if CHROME_BIN.is_none() {
            return false;
        } else {
            return true;
        }
    }
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn show_item_in_folder(path: String, dbus_state: State<DbusState>) -> Result<(), String> {
    let dbus_guard = dbus_state.0.lock().map_err(|e| e.to_string())?;

    // see https://gitlab.freedesktop.org/dbus/dbus/-/issues/76
    if dbus_guard.is_none() || path.contains(",") {
        let mut path_buf = PathBuf::from(&path);
        let new_path = match path_buf.is_dir() {
            true => path,
            false => {
                path_buf.pop();
                path_buf.into_os_string().into_string().unwrap()
            }
        };
        Command::new("xdg-open")
            .arg(&new_path)
            .spawn()
            .map_err(|e| format!("{e:?}"))?;
    } else {
        // https://docs.rs/dbus/latest/dbus/
        let dbus = dbus_guard.as_ref().unwrap();
        let proxy = dbus.with_proxy(
            "org.freedesktop.FileManager1",
            "/org/freedesktop/FileManager1",
            Duration::from_secs(5),
        );
        let (_,): (bool,) = proxy
            .method_call(
                "org.freedesktop.FileManager1",
                "ShowItems",
                (vec![format!("file://{path}")], ""),
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(())
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

// MARK: Main Function
/// Main application entry function.
fn main() {

    // Rebase all StdOut and StdErr happenings
    #[cfg(not(debug_assertions))]
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

    #[cfg(not(target_os = "linux"))]
    tauri::Builder::default()
        .menu(window_menu.clone())
        .manage(Storage::default())
        .invoke_handler(tauri::generate_handler![update_storage_data, create_wettkampf, sync_wk_data_and_open_editor, get_wk_data_to_frontend, sync_to_backend_and_save, sync_to_backend_and_create_docx, sync_to_backend_and_create_pdf, import_wk_file_and_open_editor, check_for_chrome_binary, download_chrome, check_if_pdf_is_available, show_item_in_folder])
        .setup(|_app| {
            Ok(())
        })
        .on_menu_event(move |ev| {
            // Get an AppHandle Clone
            let app_handle = ev.window().app_handle().clone();
            match ev.menu_item_id() {
                "showLicenses" => {
                    if app_handle.windows().contains_key("licenseWindow") {
                        let windows = app_handle.windows();
                        let license_window = windows.get("licenseWindow").unwrap();
                        license_window.show().unwrap();
                        license_window.set_focus().unwrap();
                        return;
                    }
                    let license_window = WindowBuilder::new(&app_handle.clone(), "licenseWindow", tauri::WindowUrl::App(PathBuf::from("licenses.html")))
                        .menu(window_menu.clone())
                        .inner_size(550.0, 600.0)
                        .title("Open Source Lizenzen".to_string())
                        .center()
                        .focused(true)
                        .visible(true)
                        .build()
                        .unwrap();
                    license_window.show().unwrap();
                }
                &_ => {}
            }
        })
        .build(tauri::generate_context!())
        .unwrap()
        .run(|_app_handle, _ev| {

        });

    #[cfg(target_os = "linux")]
    tauri::Builder::default()
        .menu(window_menu.clone())
        .manage(Storage::default())
        .manage(DbusState(Mutex::new(dbus::blocking::SyncConnection::new_session().ok())))
        .invoke_handler(tauri::generate_handler![update_storage_data, create_wettkampf, sync_wk_data_and_open_editor, get_wk_data_to_frontend, sync_to_backend_and_save, sync_to_backend_and_create_docx, sync_to_backend_and_create_pdf, import_wk_file_and_open_editor, check_for_chrome_binary, download_chrome, check_if_pdf_is_available, show_item_in_folder])
        .setup(|_app| {
            Ok(())
        })
        .on_menu_event(move |ev| {
            // Get an AppHandle Clone
            let app_handle = ev.window().app_handle().clone();
            match ev.menu_item_id() {
                "showLicenses" => {
                    if app_handle.windows().contains_key("licenseWindow") {
                        let windows = app_handle.windows();
                        let license_window = windows.get("licenseWindow").unwrap();
                        license_window.show().unwrap();
                        license_window.set_focus().unwrap();
                        return;
                    }
                    let license_window = WindowBuilder::new(&app_handle.clone(), "licenseWindow", tauri::WindowUrl::App(PathBuf::from("licenses.html")))
                        .menu(window_menu.clone())
                        .inner_size(550.0, 600.0)
                        .title("Open Source Lizenzen".to_string())
                        .center()
                        .focused(true)
                        .visible(true)
                        .build()
                        .unwrap();
                    license_window.show().unwrap();
                }
                &_ => {}
            }
        })
        .build(tauri::generate_context!())
        .unwrap()
        .run(|_app_handle, _ev| {

        });
}
