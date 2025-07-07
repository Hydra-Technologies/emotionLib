use serde::Serialize;
use crate::dosb_eval::DOSBAbzeichen;
use crate::bjs_eval::BJSAbzeichen;
use crate::model::Attempt;

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


pub struct Filter {
    pub bjs: Option<Vec<BJSAbzeichen>>,
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
pub struct SchuelerResult {
    pub id: i64,
    pub bjs_punkte: i64,
    pub bjs_urkunde: BJSAbzeichen,
    pub dosb_punkte: i64,
    pub dosb_abzeichen: DOSBAbzeichen,
}
#[derive(Debug, Serialize)]
pub struct SchuelerResultExtensive {
    pub id: i64,
    pub bjs_punkte: i64,
    pub bjs_urkunde: BJSAbzeichen,
    pub dosb_punkte: i64,
    pub dosb_abzeichen: DOSBAbzeichen,
    pub single_results: Vec<Attempt>,
}
