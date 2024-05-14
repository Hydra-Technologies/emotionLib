use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BjsAlterBewertung {
    pub alter: i8,
    pub gesch: char,
    pub ehren: i32,
    pub sieger: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DosbAlterBewertung {
    pub alter: i8,
    pub bronze: f32,
    pub silber: f32,
    pub gold: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BjsKategorieConstructor {
    pub a_w: f32,
    pub c_w: f32,
    pub a_m: f32,
    pub c_m: f32,
    pub versuche: u8,
    pub formel: String,
    pub altersklassen_m: Vec<i8>,
    pub altersklassen_w: Vec<i8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DosbKategorieConstructor {
    pub altersklassen_m: Vec<DosbAlterBewertung>,
    pub altersklassen_w: Vec<DosbAlterBewertung>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AltersklassenConstructor {
    pub altersklassen_m: Vec<i8>,
    pub altersklassen_w: Vec<i8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Kategorie {
    pub name: String,
    pub einheit: String,
    pub kat_group: i8,
    pub digits_before: i8,
    pub digits_after: i8,
    pub versuche: u8,
    pub bjs: Option<BjsKategorieConstructor>,
    pub dosb: Option<DosbKategorieConstructor>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct KategorieChanges {
    pub name: Option<String>,
    pub digits_before: Option<i8>,
    pub digits_after: Option<i8>,
    pub versuche: Option<u8>,
    pub bjs: Option<AltersklassenConstructor>, // IMPORTANT I use dosb because a and c should not be changed
    pub dosb: Option<AltersklassenConstructor>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KategorieVorlage {
    pub bjs: Option<bool>,  // if the bjs part should be inserted (none == false)
    pub dosb: Option<bool>, // if the dosb part should be inserted when both are false this Kat is skipped
    pub id: i32,
    pub changes: Option<KategorieChanges>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutsideKategorie {
    pub id: i8,
    pub name: String,
    pub einheit: String,
    pub kat_group: i8,
    pub versuche: u8,
    pub digits_before: i8,
    pub digits_after: i8,
    pub bjs: Option<BjsKategorieConstructor>,
    pub dosb: Option<DosbKategorieConstructor>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConstructKategorie {
    Vorlage(KategorieVorlage),
    Kategorie(Kategorie),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventConstructor {
    pub name: String,
    pub vorlage: i32, // Here the Year from wich the vorlage should be used is spezified, if none is given the newest is used
    pub bjs_bewertung: Option<Vec<BjsAlterBewertung>>,
    pub kategorien: Option<Vec<ConstructKategorie>>,
}
