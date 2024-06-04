use crate::search::search_schema::DOSBAbzeichen;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize)]
pub struct SimpleSchueler {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct SimpleVersuch {
    pub schueler_id: i32,
    pub wert: f32,
    pub kategorie_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadSchueler {
    pub id: i64,
    pub gesch: char,
    pub age: Option<i8>,
    pub bday: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NormVersuch {
    pub id: i64,
    pub schueler_id: i64,
    pub kategorie_id: i64,
    pub wert: f64,
    pub punkte: i64,
    pub ts_recording: i64,
    pub is_real: bool,
}

#[derive(Debug, Serialize)]
pub struct NormVersuchDosb {
    pub id: i64,
    pub schueler_id: i64,
    pub kategorie_id: i64,
    pub wert: f64,
    pub punkte: i64,
    pub dosb: DOSBAbzeichen,
    pub ts_recording: i64,
    pub is_real: bool,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PflichtKategorie {
    pub id: i64,
    pub done: bool,
    pub group_id: i64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct SimpleKategorie {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Kategorie {
    pub id: i64,
    pub name: String,
    pub lauf: bool,
    pub einheit: char,
    pub max_vers: i64,
    pub digits_before: i64,
    pub digits_after: i64,
    pub kat_group_id: i64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct NeedsKat {
    pub dosb: bool,
    pub bjs: bool,
}
