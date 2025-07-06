use serde::Serialize;

pub enum SortKategorien {
    Age,
    Klasse,
    FirstName,
    LastName,
}

pub enum SearchKategorien {
    Klasse,
    Name,
}

#[derive(Debug, Serialize)]
pub enum BJSUrkunde {
    Teilnehmer,
    Sieger,
    Ehren,
    None,
}


pub struct Filter {
    pub bjs: Option<Vec<BJSUrkunde>>,
    pub dosb: Option<Vec<DOSBAbzeichen>>,
}

pub struct SearchTerm {
    pub term: Option<String>,
    pub kat: Option<SearchKategorien>,
    pub filter: Option<Filter>,
    pub sort: Option<SortKategorien>,
    pub extensive: bool,
}

#[derive(Debug, Serialize)]
pub struct SingleResult {
    pub kategorie_id: i64,
    pub wert: f64,
}

#[derive(Debug, Serialize)]
pub struct SchuelerResult {
    pub id: i64,
    pub bjs_punkte: i64,
    pub bjs_urkunde: BJSUrkunde,
    pub dosb_punkte: i64,
    pub dosb_abzeichen: DOSBAbzeichen,
}
#[derive(Debug, Serialize)]
pub struct SchuelerResultExtensive {
    pub id: i64,
    pub bjs_punkte: i64,
    pub bjs_urkunde: BJSUrkunde,
    pub dosb_punkte: i64,
    pub dosb_abzeichen: DOSBAbzeichen,
    pub single_results: Vec<SingleResult>,
}
