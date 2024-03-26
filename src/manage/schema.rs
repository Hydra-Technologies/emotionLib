use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BjsAlterBewertung {
    pub alter: i8,
    pub gesch: char,
    pub ehren: i32,
    pub sieger: i32
}

#[derive(Serialize, Deserialize)]
pub struct DosbAlterBewertung {
    pub alter: i8,
    pub bronze: f32,
    pub silber: f32,
    pub gold: f32
}

#[derive(Serialize, Deserialize)]
pub struct BjsKategorieConstructor {
    pub a_w: f32,
    pub c_w: f32,
    pub a_m: f32,
    pub c_m: f32,
    pub formel: String,
    pub altersklassen_m: Vec<i8>,
    pub altersklassen_w: Vec<i8>,
}

#[derive(Serialize, Deserialize)]
pub struct DosbKategorieConstructor {
    pub altersklassen_m: Vec<DosbAlterBewertung>,
    pub altersklassen_w: Vec<DosbAlterBewertung>
}
#[derive(Serialize, Deserialize)]

pub struct Kategorie {
    pub name: String,
    pub einheit: String,
    pub kat_group: i8,
    pub digits_before: i8,
    pub digits_after: i8,
    pub bjs: BjsKategorieConstructor,
    pub dosb: DosbKategorieConstructor
}

#[derive(Serialize, Deserialize)]
pub struct KategorieVorlage {
    pub bjs: Option<bool>, // if the bjs part should be inserted (none == false)
    pub dosb: Option<bool>, // if the dosb part should be inserted when both are false this Kat is skipped
    pub id: i32,
    pub changes: Kategorie
}
#[derive(Serialize, Deserialize)]
pub enum ConstructKategorie {
    Vorlage(KategorieVorlage),
    Kategorie(Kategorie)
}

#[derive(Serialize, Deserialize)]
pub struct EventConstructor {
    pub name: String,
    pub vorlage: Option<i32>, // Here the Year from wich the vorlage should be used is spezified, if none is given the newest is used
    pub bjs_bewertung: Vec<BjsAlterBewertung>,
    pub kategorien: Option<Vec<ConstructKategorie>>
}
