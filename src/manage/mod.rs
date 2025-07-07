pub mod schema;

use log::{debug, info, warn};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::string::String;
use std::ffi::OsStr;
use walkdir;
use actix_web::HttpResponse;
use crate::{InternalServer,Conflict,NotFound};

pub async fn create_event(
    school_dir: String,
    vorlagen_dir: String,
    id: String,
    data: schema::EventConstructor,
) -> Result<SqlitePool, HttpResponse> {
    let db_url = format!("{}{}.db", school_dir, id);
    // check if database exists
    if Sqlite::database_exists(db_url.as_str())
        .await
        .unwrap_or(true)
    {
        return Err(Conflict!("Event Alread exists (or error)"));
    }
    info!("checked if db exists");

    // create Database
    match Sqlite::create_database(db_url.as_str()).await {
        Err(e) => {
            info!("{}", e.to_string());
            return Err(InternalServer!(format!("Something went wrong while creating Table ({})", e)));
        },
        Ok(_) => (),
    };
    info!("created DB");

    let con = SqlitePool::connect(db_url.as_str()).await.unwrap();

    // write the database schema to the file
    sqlx::migrate!("./event_migrations").run(&con).await.unwrap();

    info!("migrated DB");
    if data.kategorien.is_some() {
        for kat in data.kategorien.unwrap() {
            let _ = match kat {
                // TODO This will not work!
                schema::ConstructKategorie::Kategorie(_) => panic!("Kategorien werden nicht mehr supported"),//insert_kat_in_db(&con, k).await,
                // only ever use vorlagen
                schema::ConstructKategorie::Vorlage(v) => {
                    insert_kat_in_db(
                        &con,
                        get_kat_from_vorlage(vorlagen_dir.clone(), 2025, v.id)?
                    )
                    .await
                }
            };
        }
    }

    return Ok(con);
}

pub fn get_vorlagen(vorlagen_path: String) -> Vec<String> {
    let vorlagen_path = Path::new(&vorlagen_path);
    let mut vorlagen = vec![];
    if vorlagen_path.is_dir() {
        for entry in fs::read_dir(vorlagen_path).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                let filename = entry.file_name().to_str().unwrap().to_string();
                if filename.parse::<i32>().is_ok() {
                    vorlagen.push(filename);
                }
            }
        }
    }
    return vorlagen;
}

pub fn get_kat_list_from_vorlage(
    vorlagen_path: String,
    year: i32,
) -> Result<Vec<schema::OutsideKategorie>, HttpResponse> {
    let mut kat_list: Vec<schema::OutsideKategorie> = vec![];
    for entry_result in walkdir::WalkDir::new([vorlagen_path, year.to_string()].join("")) {
        if let Ok(entry) = entry_result {
            debug!("Path: {:?}", entry.path());
            if entry.file_name() != "init.json" {
                if let Ok(k) = serde_json::from_reader::<File, schema::Kategorie>(
                    File::open(entry.path()).unwrap(),
                ) {
                    kat_list.push(schema::OutsideKategorie {
                        id: entry
                            .file_name()
                            .to_string_lossy()
                            .split(".")
                            .into_iter()
                            .find(|e| e.to_string().parse::<i8>().is_ok())
                            .unwrap()
                            .to_string()
                            .parse()
                            .unwrap(),
                        name: k.name,
                        einheit: k.einheit,
                        kat_group: k.kat_groupBJS,
                        digits_before: k.digits_before,
                        digits_after: k.digits_after,
                        versuche: k.versuche,
                        bjs: k.bjs,
                        dosb: k.dosb,
                    });
                }
            }
        }
    }
    return Ok(kat_list);
}
/**
 * checks the given Vorlagen Path for Syntax error and if the all the Attributes are there
 */
pub fn check_vorlagen(vorlagen_dir: String) -> Result<(), String> {
    let vorlagen = walkdir::WalkDir::new(vorlagen_dir);
    // walks through the json files in the vorlage
    for vorlage in vorlagen {
        // if it checksout
        if vorlage.is_err() {
            continue;
        }
        let dir = vorlage.unwrap();


        // and it is a file
        if dir.path().is_dir() {
            continue;
        }

        // and it is a json
        if dir.path().extension().unwrap() != OsStr::new("json") {
            continue;
        }

        // then open it.
        let file = File::open(dir.path()).unwrap();
        let reader = std::io::BufReader::new(file);

        // if it is the init
        if dir.file_name().eq("init.json") {
            // then check if you can open it
            let _: schema::EventConstructor = match serde_json::from_reader(reader) {
                Err(e) => {
                    return Err(format!(
                            "Couldnt read init file {}: {}",
                            dir.path().to_str().unwrap(),
                            e.to_string()
                    ))
                }
                Ok(r) => r,
            };
        } else {
            // else it must be a Kategorien file 
            // check if it is in the right format
            let _: schema::Kategorie = match serde_json::from_reader(reader) {
                Err(e) => {
                    return Err(format!(
                            "Couldnt read Kategorie file {}: {}",
                            dir.path().to_str().unwrap(),
                            e.to_string()
                    ))
                }
                Ok(r) => r,
            };
        }
    }
    return Ok(());
}

pub fn get_kat_from_vorlage(
    vorlagen_dir: String,
    year: i32,
    kat_id: i32,
) -> Result<schema::Kategorie, HttpResponse> {
    let path_string = format!("{}{}/{}.json", vorlagen_dir, year, kat_id);

    let path = Path::new(&path_string);
    let reader = match File::open(path) {
        Err(_) => {
            return Err(NotFound!(format!( "Kategorie {:?} konnte nicht gefunden werden", path)));
        }
        Ok(f) => std::io::BufReader::new(f),
    };

    match serde_json::from_reader(reader) {
        Err(_) => {
            return Err(InternalServer!("Error while reading Kategorie"))
        },
        Ok(r) => Ok(r),
    }
}

pub fn get_kat_by_vorlage(
    vorlagen_path: String,
    vorlage: i64,
) -> Result<Vec<schema::Kategorie>, HttpResponse> {
    let files = match fs::read_dir([vorlagen_path, vorlage.to_string(), "/".to_string()].join("")) {
        Ok(p) => p,
        Err(_) => {
            return Err(InternalServer!("Vorlagen Dir not found"))
        }
    };

    let mut kategorien: Vec<schema::Kategorie> = vec![];
    for file in files {
        match file.unwrap() {
            f => {
                let name_vec: Vec<String> = f
                    .file_name()
                    .to_str()
                    .unwrap()
                    .split(".")
                    .map(|s| s.to_string())
                    .collect();
                let id = name_vec[0].parse::<i32>();
                if id.is_ok() {
                    if let Ok(reader) = File::open(f.path()) {
                        let kat: schema::Kategorie = match serde_json::from_reader(reader) {
                            Ok(k) => k,
                            Err(_) => {
                                return Err(InternalServer!("Id nicht gefunden"))
                            }
                        };
                        kategorien.push(kat);
                    } else {
                        warn!(
                            "Couldnt read File: {}",
                            f.path().to_str().unwrap_or("Couldnt unwrap Path")
                        );
                    };
                }
            }
        }
    }
    return Ok(kategorien);
}

async fn insert_kat_in_db(db: &SqlitePool, kat: schema::Kategorie) -> Result<(), HttpResponse> {
    match sqlx::query!("INSERT INTO kategorien(name, einheit, maxVers, digits_before, digits_after) VALUES (?,?,?,?,?)",
        kat.name, kat.einheit, kat.versuche, kat.digits_before, kat.digits_after).execute(db).await {
        Ok(r) => r.last_insert_rowid(),
        Err(_e) => return Err(InternalServer!("Error while inserting into Kategorien"))
    };
    info!("Inserted basic");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_vorlagen() {
        check_vorlagen("vorlagen/".to_string()).unwrap();
    }

    #[test]
    pub fn get_list_of_events() {
        let k_r = get_kat_list_from_vorlage("vorlagen/".to_string(), 2023);
        let k = k_r.unwrap();
        println!("{:?}", k);
    }
}
