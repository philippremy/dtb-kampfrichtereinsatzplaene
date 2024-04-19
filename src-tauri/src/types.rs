use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Kampfrichter {
    pub role: String,
    pub name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Kampfgericht {
    pub table_name: String,
    pub table_kind: String,
    pub table_is_finale: bool,
    pub judges: Vec<Kampfrichter>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub wk_name: Mutex<String>,
    pub wk_date: Mutex<String>,
    pub wk_place: Mutex<String>,
    pub wk_responsible_person: Mutex<String>,
    pub wk_judgesmeeting_time: Mutex<String>,
    pub wk_replacement_judges: Mutex<Vec<String>>,
    pub wk_judgingtables: Mutex<Vec<Kampfgericht>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FrontendStorage {
    pub wk_name: String,
    pub wk_date: String,
    pub wk_place: String,
    pub wk_responsible_person: String,
    pub wk_judgesmeeting_time: String,
    pub wk_replacement_judges: Option<Vec<String>>,
    pub wk_judgingtables: Option<Vec<Kampfgericht>>,
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
}