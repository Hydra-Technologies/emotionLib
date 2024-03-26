mod schema;

use std::fs::File;
use std::os::raw::c_schar;
use std::path::Path;
use actix_web::http::uri::Scheme;
use sqlx::{sqlite, Sqlite, SqlitePool};
use actix_web::HttpResponse;
use sqlx::migrate::MigrateDatabase;
use walkdir;
use std::string::String;

pub async fn create_event(school_dir: String, data: schema::EventConstructor) -> Result<(), HttpResponse> {
    let db_url = [school_dir, data.name.clone(), ".db".to_string()].join("");
    // check if database exists
    if Sqlite::database_exists(db_url.as_str()).await.unwrap_or(true) {
        return Err(HttpResponse::Conflict().json(serde_json::json!({"message": "Event Alread exists (or error)"})));
    }

    // create Database
    if Sqlite::create_database(db_url.as_str()).await.is_err() {
        return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Something went wrong while creating Table"})));
    }

    let con = SqlitePool::connect(db_url.as_str()).await.unwrap();

    // write the database schema to the file
    sqlx::migrate!("./migrations")
        .run(&con)
        .await.unwrap();
    return Ok(());
}

pub fn check_vorlagen(vorlagen_dir: String ) -> Result<(), String> {
    let vorlagen = walkdir::WalkDir::new(vorlagen_dir);
    for vorlage in vorlagen {
        if vorlage.is_ok() {
            let dir = vorlage.unwrap();
            if !dir.path().is_dir() {
                let file = File::open(dir.path()).unwrap();
                let reader = std::io::BufReader::new(file);
                if dir.file_name().eq("init.json") {
                    let _: schema::EventConstructor = match serde_json::from_reader(reader) {
                        Err(e) => return Err(format!("Couldnt read init file {}: {}", dir.path().to_str().unwrap(), e.to_string())),
                        Ok(r) => r
                    };
                } else {
                    let _: schema::Kategorie = match serde_json::from_reader(reader) {
                        Err(e) => return Err(format!("Couldnt read Kategorie file {}: {}", dir.path().to_str().unwrap(), e.to_string())),
                        Ok(r) => r
                    };
                }
            }
        }
    }
    return Ok(());
}

fn get_kat_from_vorlage(vorlagen_dir: String, year: i32, vorlage: schema::KategorieVorlage) -> Result<schema::Kategorie, HttpResponse> {
    let path_string = [vorlagen_dir, year, vorlage.id, ".json"].join("");

    let path = Path::new(&path_string);
    let reader = match File::open(path).unwrap() {
        Err(_) => {
            let message = format!("Kategorie {} konnte nicht gefunden werden", path.to_str().unwrap());
            return Err(HttpResponse::NotFound().json(serde_json::json!({"message": message})));
        },
        Ok(f) => std::io::BufReader::new(f)
    };

    return match serde_json::from_reader(reader) {
        Err(e) => Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while reading Kategorie"}))),
        Ok(r) => Ok(r)
    };
}

/*
fn insert_kat_in_db(db: &SqlitePool, kat: schema::Kategorie) {
    let form = if kat.einheit == "m" { format!("{{};m}, {{}; cm}", kat.digits_before, kat.digits_after) } else {}
    if sqlx::query!("INSERT INTO kategorien(name, einheit, maxVers, messungsForm, kateGroupId) VALUES (?,?,100,?,?)",
    kat.name, kat.einheit, ) {

    }
}*/

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_vorlagen() {
        check_vorlagen("vorlagen/".to_string()).unwrap();
    }
}