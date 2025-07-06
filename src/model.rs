use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Attempt {
    pub category: i64,
    pub result: f64
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Category {
    pub id: i64,
    pub group_id: i64
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct SimpleSchueler {
    pub id: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}


#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct PflichtKategorie {
    pub id: Option<i64>,
    pub done: i64,
    pub group_id: Option<i64>
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct KatGroup {
    pub id: Option<i64>,
    pub group_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct SimpleKategorie {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct NormVersuch {
    pub id: Option<i64>,
    pub schueler_id: Option<i64>,
    pub kategorie_id: Option<i64>,
    pub wert: Option<f64>,
    pub ts_recording: Option<i64>,
    pub is_real: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct FormelKategorie {
    pub name: Option<String>,
    pub a: Option<f64>,
    pub c: Option<f64>,
    pub lauf: Option<bool>
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct VersuchId {
    pub id: Option<i64>
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct KatId {
    pub id: Option<i64>
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Kategorie {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub einheit: Option<String>,
    pub max_vers: Option<i64>,
    pub digits_before: Option<i64>,
    pub digits_after: Option<i64>,
}
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct NeedsKat {
    pub need: i32,
}
