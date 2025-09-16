#![allow(unused_imports)]

use std::path::PathBuf;
use objc2::AllocAnyThread;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{types::ApplicationError, PlatformWebViewWrapper};

#[cfg(target_os = "macos")]
impl PlatformWebViewWrapper {

    pub(crate) fn print_pdf(&self, html_file_path: PathBuf, pdf_output_path: PathBuf, _app_handle: AppHandle, generated_docx: String, generated_html: String) -> ApplicationError {

        use objc2::{runtime::ProtocolObject, ClassType};
        use objc2_app_kit::{NSPrintInfo, NSPrintJobDisposition, NSPrintJobSavingURL, NSPrintSaveJob};
        use objc2_foundation::{NSMutableDictionary, NSString, NSURL};
        use objc2_web_kit::WKWebView;

        unsafe {
            if let Some(platform_webview) = &self.inner {

                let webview_ptr = platform_webview.inner();
                let webview_ptr_cast: *mut WKWebView = webview_ptr.cast();
                if webview_ptr_cast.is_null() {
                    return ApplicationError::UnknownError;
                }
                let webview: &WKWebView = &webview_ptr_cast.as_ref().unwrap();

                // Allocations
                let file_url_alloc = NSURL::alloc();
                let file_url = NSURL::initFileURLWithPath(file_url_alloc, &NSString::from_str(html_file_path.to_str().unwrap()));
                let _load_obj = webview.loadFileURL_allowingReadAccessToURL(&file_url, &file_url).unwrap();
                while webview.isLoading() {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }

                // Allocate the output URL
                let file_out_url_alloc = NSURL::alloc();
                let file_out_url = NSURL::initFileURLWithPath(file_out_url_alloc, &NSString::from_str(pdf_output_path.to_str().unwrap()));

                // Set the important dictionary entries
                let mut print_info_dict = NSMutableDictionary::new();
                print_info_dict.setValue_forKey(Some(NSPrintSaveJob.as_ref() as _), NSPrintJobDisposition);
                print_info_dict.setObject_forKey(file_out_url.as_ref() as _, &ProtocolObject::from_ref(NSPrintJobSavingURL));

                // Create the PDF Print Info
                let ns_printinfo_alloc = NSPrintInfo::alloc();
                let ns_printinfo = NSPrintInfo::initWithDictionary(ns_printinfo_alloc, &print_info_dict);
                ns_printinfo.setPaperName(Some(&NSString::from_str("iso-a4")));
                ns_printinfo.setBottomMargin(0.0);
                ns_printinfo.setTopMargin(0.0);
                ns_printinfo.setLeftMargin(0.0);
                ns_printinfo.setRightMargin(0.0);
                ns_printinfo.setScalingFactor(1.0);
                ns_printinfo.setHorizontallyCentered(true);
                ns_printinfo.setVerticallyCentered(true);

                // Create print operation
                let ns_print_operation = webview.printOperationWithPrintInfo(&ns_printinfo);
                ns_print_operation.setShowsPrintPanel(false);
                ns_print_operation.setShowsProgressPanel(false);

                // Print
                let success = ns_print_operation.runOperation();
                if !success {
                    println!("Failed to print to PDF, 'runOperation()' returned NO");
                    return ApplicationError::WritingPDFDataToDiskFailed;
                }

            }
        }

        // Delete temporary files
        match std::fs::remove_file(generated_html) {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Could not remove temporary generated HTML file: {:?}", err);
            }
        }

        // Delete temporary files
        match std::fs::remove_file(generated_docx) {
            Ok(()) => {}
            Err(err) => {
                println!("Could not remove temporary generated DOCX file: {:?}", err);
            }
        }

        return ApplicationError::NoError;

    }

}

#[cfg(target_os = "windows")]
#[derive(Clone, Serialize, Deserialize)]
struct PdfCreationResponseWin {
    operation_succeeded: bool
}

#[cfg(target_os = "windows")]
impl PlatformWebViewWrapper {

    pub(crate) fn print_pdf(&self, html_file_path: PathBuf, pdf_output_path: PathBuf, app_handle: AppHandle, generated_docx: String, generated_html: String) -> ApplicationError {

        use tauri::Emitter;
        use windows_core::Interface;
        use webview2_com::{pwstr_from_str, Microsoft::Web::WebView2::Win32::{ICoreWebView2Environment6, ICoreWebView2_2, ICoreWebView2_7}, NavigationCompletedEventHandler, PrintToPdfCompletedHandler};

        unsafe {

            // Get the WebView
            if let Some(platform_webview) = &self.inner {

                // Fetch the WebViews
                let core_webview = platform_webview.controller().CoreWebView2().unwrap();
                let core_webview_7: ICoreWebView2_7 = core_webview.cast::<ICoreWebView2_7>().unwrap();

                // Fetch the environment
                let core_webview_2: ICoreWebView2_2 = core_webview.cast::<ICoreWebView2_2>().unwrap();
                let webview_environment = core_webview_2.Environment().unwrap();
                let webview_enviornment_6: ICoreWebView2Environment6 = webview_environment.cast::<ICoreWebView2Environment6>().unwrap();

                // Navigate to the local HTML
                let mut event_token = 0i64;
                let navigation_event_handler = NavigationCompletedEventHandler::create(Box::new(move |_, _| {

                    // Continue in here, the completion handler runs on the same thread
                    // and waiting atomically will cause a deadlock

                    // Generate PDF Printing Options
                    let print_settings = webview_enviornment_6.CreatePrintSettings().unwrap();
                    print_settings.SetMarginBottom(0.0).unwrap();
                    print_settings.SetMarginTop(0.0).unwrap();
                    print_settings.SetMarginLeft(0.0).unwrap();
                    print_settings.SetMarginRight(0.0).unwrap();
                    print_settings.SetShouldPrintHeaderAndFooter(false).unwrap();
                    print_settings.SetShouldPrintSelectionOnly(false).unwrap();
                    print_settings.SetPageHeight(11.67).unwrap();
                    print_settings.SetPageWidth(8.27).unwrap();

                    // Generate PCWSTR
                    let out_pcwstr = pwstr_from_str(pdf_output_path.to_str().unwrap());

                    // Clone AppHandle to pass further down the callback chain
                    let app_handle_inner_box = app_handle.clone();
                    let generated_docx_inner_box = generated_docx.clone();
                    let generated_html_inner_box = generated_html.clone();

                    // Generate Completion Handler
                    let print_event_handler = PrintToPdfCompletedHandler::create(Box::new(move |result, success| {

                        // Print error, if we errored
                        if !success {
                            eprintln!("Failed to save PDF: HWResult was {result:?}");
                        }

                        // Delete temporary files
                        match std::fs::remove_file(generated_html_inner_box) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("Could not remove temporary generated HTML file: {:?}", err);
                            }
                        }

                        // Delete temporary files
                        match std::fs::remove_file(generated_docx_inner_box) {
                            Ok(()) => {}
                            Err(err) => {
                                println!("Could not remove temporary generated DOCX file: {:?}", err);
                            }
                        }

                        // Give answer to the frontend from here
                        app_handle_inner_box.emit("pdfCreationFinishedWindows", PdfCreationResponseWin { operation_succeeded: success }).unwrap();

                        Ok(())
                    }));

                    // Print
                    core_webview_7.PrintToPdf(out_pcwstr, &print_settings, &print_event_handler).unwrap();

                    Ok(())
                }));

                // Navigate
                core_webview.add_NavigationCompleted(&navigation_event_handler, std::ptr::from_mut(&mut event_token) as _).unwrap();
                core_webview.Navigate(pwstr_from_str(format!("file://{}", html_file_path.to_str().unwrap()).as_str())).unwrap();

            }

        }

        return ApplicationError::WaitingForWindowsPDFResult;

    }

}
