use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Mutex};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Kampfrichter {
    pub role: String,
    pub name: String,
    pub doubleFound: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Kampfgericht {
    pub uniqueID: String,
    pub table_name: String,
    pub table_kind: String,
    pub table_is_finale: bool,
    pub judges: HashMap<String, Kampfrichter>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub wk_name: Mutex<String>,
    pub wk_date: Mutex<String>,
    pub wk_place: Mutex<String>,
    pub wk_responsible_person: Mutex<String>,
    pub wk_judgesmeeting_time: Mutex<String>,
    pub wk_replacement_judges: Mutex<Vec<String>>,
    pub wk_judgingtables: Mutex<HashMap<String, Kampfgericht>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FrontendStorage {
    pub wk_name: String,
    pub wk_date: String,
    pub wk_place: String,
    pub wk_responsible_person: String,
    pub wk_judgesmeeting_time: String,
    pub wk_replacement_judges: Option<Vec<String>>,
    pub wk_judgingtables: Option<HashMap<String, Kampfgericht>>,
}

#[repr(C)]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum ApplicationError {
    UnknownError = -1,
    NoError = 0,
    MutexPoisonedError = 1,
    JSONSerializeError = 2,
    CStringNullError = 3,
    MarshalJSONNullError = 4,
    DeserializeArgumentNullError = 5,
    DeserializeJSONError = 6,
    DeserializeNotSupportedError = 7,
    TauriWindowCreationError = 8,
    TauriWindowShowError = 9,
    RustWriteFileError = 10,
    MarshalSavePathNullError = 11,
    StorageNullError = 12,
    CSharpWriteError = 13,
    JSONDeserializeImporterError = 14,
    FailedToCreateStdOutFileError = 15,
    FailedToCreateStdErrFileError = 16,
    LibcDup2StdOutError = 17,
    LibcDup2StdErrError = 18,
    CSharpPDFSavePathIsEmpty = 19,
    ChromeDownloadError = 20,
    ChromiumBinaryIsUnexpectedlyNone = 21,
    BrowserCouldNotBeBuild = 22,
    NewTabCouldNotBeCreated = 23,
    NavigationToGeneratedHTMLFileFailed = 24,
    WaitingForNavigationFailed = 25,
    PDFGenerationInChromiumFailed = 26,
    WritingPDFDataToDiskFailed = 27,
    RemovalOfTemporaryGeneratedFilesFailed = 28,
    SMTPConnectionError = 29,
    MessageSendError = 30,
    TauriExistingWindowNotFoundError = 31,
    WaitingForWindowsPDFResult = 32
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateAvailablePayload {
    pub(crate) body: String,
    pub(crate) date: String,
    pub(crate) version: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateProgressPayload {
    pub(crate) chunk_len: usize,
    pub(crate) content_len: Option<u64>,
}
