use std::path::PathBuf;
use crate::{types::ApplicationError, PlatformWebViewWrapper};

#[cfg(target_os = "macos")]
impl PlatformWebViewWrapper {

    pub(crate) fn print_pdf(&self, html_file_path: PathBuf, pdf_output_path: PathBuf) -> ApplicationError {

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

        return ApplicationError::NoError;

    }

}

#[cfg(target_os = "windows")]
impl PlatformWebViewWrapper {

    pub(crate) fn print_pdf(&self, html_file_path: PathBuf, pdf_output_path: PathBuf) -> ApplicationError {

        return ApplicationError::NoError;

    }

}