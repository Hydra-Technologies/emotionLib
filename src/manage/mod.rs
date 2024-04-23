pub mod schema;

use std::fs;
use std::fs::File;
use std::path::Path;
use sqlx::{Sqlite, SqlitePool};
use sqlx::migrate::MigrateDatabase;
use walkdir;
use std::string::String;
use crate::manage::schema::{BjsAlterBewertung, EventConstructor, Kategorie };
use log::{debug, info, warn};

#[derive(Debug)]
pub enum ManageError {
    Internal { message: String },
    Conflict { message: String },
    NotFound { message: String },
    BadReque { message: String }
}

pub async fn create_event(school_dir: String, vorlagen_dir: String, data: schema::EventConstructor) -> Result<(), ManageError> {
    let db_url = [school_dir, data.name.clone(), ".db".to_string()].join("");
    // check if database exists
    if Sqlite::database_exists(db_url.as_str()).await.unwrap_or(true) {
        return Err(ManageError::Conflict { message: "Event Alread exists (or error)".to_string() });
    }
    info!("checked if db exists ");

    // create Database
    match Sqlite::create_database(db_url.as_str()).await {
        Err(e) =>  {
            info!("{}",e.to_string());
            return Err(ManageError::Internal { message: ["Something went wrong while creating Table".to_string(), e.to_string()].join("") })},
        Ok(_) => ()
    };
    info!("created DB");

    let con = SqlitePool::connect(db_url.as_str()).await.unwrap();

    // write the database schema to the file
    sqlx::migrate!("./migrations")
        .run(&con)
        .await.unwrap();

    info!("migrated DB");
    if data.bjs_bewertung.is_some() {
        insert_bjs_bewertungen(&con, data.bjs_bewertung.unwrap()).await?;
    } else {
        let path = [vorlagen_dir.clone(), data.vorlage.unwrap().to_string(), "/init.json".to_string()].join("");
        let reader = std::io::BufReader::new(File::open(path).unwrap());
        let bjs: EventConstructor = serde_json::from_reader(reader).unwrap();

        insert_bjs_bewertungen(&con, bjs.bjs_bewertung.unwrap()).await?;
    }

    if data.kategorien.is_some() {
        for kat in data.kategorien.unwrap() {
            match kat {
                schema::ConstructKategorie::Kategorie(k) => insert_kat_in_db(&con, k).await,
                schema::ConstructKategorie::Vorlage(v) =>  {
                    match data.vorlage {
                        Some(y) => insert_kat_in_db(&con, get_kat_from_vorlage(vorlagen_dir.clone(), y, v)?).await,
                        None => return Err(ManageError::BadReque { message: "No Vorlage specified".to_string() })
                    }
                }
            }?
        }
    }

    return Ok(());
}

pub fn get_vorlagen(vorlagen_path: String) -> Vec<String> {
    let vorlagen_path = Path::new(&vorlagen_path);
    let mut vorlagen= vec![];
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

pub fn get_kat_list_from_vorlage(vorlagen_path: String, year: i32) -> Result<Vec<schema::Kategorie>, ManageError> {
    let mut kat_list: Vec<schema::Kategorie> = vec![];
    for entry_result in walkdir::WalkDir::new([vorlagen_path, year.to_string()].join("")) {
        if let Ok(entry) = entry_result {
            debug!("Path: {:?}", entry.path());
            if entry.file_name() != "init.json" {
                if let Ok(k) = serde_json::from_reader(File::open(entry.path()).unwrap()) {
                    kat_list.push(k);
                }
            }
        }
    }
    return Ok(kat_list);
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

pub fn get_kat_from_vorlage(vorlagen_dir: String, year: i32, vorlage: schema::KategorieVorlage) -> Result<schema::Kategorie, ManageError> {
    let path_string = [vorlagen_dir, year.to_string(), "/".to_string(), vorlage.id.to_string(), ".json".to_string()].join("");

    let path = Path::new(&path_string);
    let reader = match File::open(path) {
        Err(_) => {
            let message = format!("Kategorie {} konnte nicht gefunden werden", path.to_str().unwrap());
            return Err(ManageError::NotFound { message });
        },
        Ok(f) => std::io::BufReader::new(f)
    };

    let kat: Kategorie =  match serde_json::from_reader(reader) {
        Err(_) => return Err(ManageError::Internal{ message: "Error while reading Kategorie".to_string()}),
        Ok(r) => r
    };

    return Ok(if vorlage.changes.is_some() {
        let changes = vorlage.changes.unwrap();
        schema::Kategorie {
            name: changes.name.unwrap_or(kat.name),
            einheit: changes.einheit.unwrap_or(kat.einheit),
            kat_group: changes.kat_group.unwrap_or(kat.kat_group),
            digits_before: changes.digits_before.unwrap_or(kat.digits_before),
            digits_after: changes.digits_after.unwrap_or(kat.digits_after),
            bjs: if changes.bjs.is_some() { changes.bjs } else if vorlage.bjs.unwrap_or(false) { kat.bjs } else { None },
            dosb: if changes.dosb.is_some() { changes.dosb } else if vorlage.dosb.unwrap_or(false) { kat.dosb } else { None }
        }
    } else {
        schema::Kategorie {
            name: kat.name,
            einheit: kat.einheit,
            kat_group: kat.kat_group,
            digits_before: kat.digits_before,
            digits_after: kat.digits_after,
            bjs: if vorlage.bjs.unwrap_or(false) { kat.bjs } else { None },
            dosb: if vorlage.dosb.unwrap_or(false) { kat.dosb } else { None },
        }
    });
}

pub fn get_kat_by_vorlage(vorlagen_path: String, vorlage: i64) -> Result<Vec<schema::Kategorie>, ManageError> {
    let files = match fs::read_dir([vorlagen_path, vorlage.to_string(), "/".to_string()].join("")) {
        Ok(p) => p,
        Err(_) => return Err(ManageError::Internal{ message: "Vorlagen Dir not found".to_string()})
    };

    let mut kategorien: Vec<schema::Kategorie> = vec![];
    for file in files {
        match file.unwrap() {
            f => {
                let name_vec: Vec<String> = f.file_name().to_str().unwrap().split(".").map(|s| s.to_string()).collect();
                let id = name_vec[0].parse::<i32>();
                if id.is_ok() {
                    if let Ok(reader) = File::open(f.path()) {
                        let kat: schema::Kategorie = match serde_json::from_reader(reader) {
                            Ok(k) => k,
                            Err(_) => return Err(ManageError::Internal{ message: "Id nicht gefunden".to_string() })
                        };
                        kategorien.push(kat);
                    } else {
                        warn!("Couldnt read File: {}", f.path().to_str().unwrap_or("Couldnt unwrap Path"));
                    };
                }
            }
        }
    }
    return Ok(kategorien);
}


async fn insert_bjs_bewertungen(db: &SqlitePool, bjs_alter_bewertungen: Vec<BjsAlterBewertung>) -> Result<(), ManageError> {
    for bew in bjs_alter_bewertungen {
        let gesch_str = bew.gesch.to_string();
        if sqlx::query!("INSERT INTO ageGroups(age,gesch, gold, silber) VALUES (?,?,?,?)", bew.alter, gesch_str , bew.ehren, bew.sieger).execute(db).await.is_err() {
            return Err(ManageError::Internal{ message: "Error while inserting bjs bewertungen".to_string() });
        }
    }
    Ok(())
}

async fn insert_kat_in_db(db: &SqlitePool, kat: schema::Kategorie) -> Result<(), ManageError>{
    let lauf = kat.kat_group == 1 || kat.kat_group == 4;
    let id = match sqlx::query!("INSERT INTO kategorien(name, einheit, lauf, maxVers, digits_before, digits_after, kateGroupId) VALUES (?,?,?,100,?,?,?)",
        kat.name, kat.einheit, lauf, kat.digits_before, kat.digits_after, kat.kat_group).execute(db).await {
        Ok(r) => r.last_insert_rowid(),
        Err(_e) => return Err(ManageError::Internal{ message: "Error while inserting into Kategorien".to_string() })
    };

    info!("Inserted basic");

    // inserting BJS
    if kat.bjs.is_some() {
        let bjs = kat.bjs.unwrap();

        // the a and c numbers
        if sqlx::query!("INSERT INTO formVars(katId, gesch, a, c) VALUES (?,'w',?,?)", id, bjs.a_w, bjs.c_w).execute(db).await.is_err() {
            return Err(ManageError::Internal{ message: "Error while inserting w a and c".to_string() });
        }
        if sqlx::query!("INSERT INTO formVars(katId, gesch, a, c) VALUES (?,'m',?,?)", id, bjs.a_m, bjs.c_m).execute(db).await.is_err() {
            return Err(ManageError::Internal{ message: "Error while inserting m a and c".to_string()});
        }

        // the age groups
        for age in bjs.altersklassen_w {
            match sqlx::query!("INSERT INTO bjsKat(katId, gesch, age) VALUES (?, 'w', ?)", id, age).execute(db).await {
                Err(e) => {
                    info!("{}", e);
                    return Err(ManageError::Internal { message: "Error while inserting w alterklassen ".to_string()});
                },
                Ok(_) => ()
            }
        }


        for age in bjs.altersklassen_m {
            if sqlx::query!("INSERT INTO bjsKat(katId, gesch, age) VALUES (?, 'm', ?)", id, age).execute(db).await.is_err() {
                return Err(ManageError::Internal { message: "Error while inserting m alterklassen".to_string()});
            }
        }
    }

    info!("Inserted BJS");

    // DOSB
    if kat.dosb.is_some() {
        let dosb = kat.dosb.unwrap();

        // Insert altersklasseen
        for age_bew in dosb.altersklassen_w {
            if sqlx::query!("INSERT INTO dosbKat(katId, gesch, age, gold, silber, bronze) VALUES (?, 'w', ?, ?, ?, ?)", id, age_bew.alter, age_bew.gold, age_bew.silber, age_bew.bronze).execute(db).await.is_err() {
                return Err(ManageError::Internal{ message: "Error while inserting w alterklassen".to_string()});
            }
        }

        for age_bew in dosb.altersklassen_m {
            if sqlx::query!("INSERT INTO dosbKat(katId, gesch, age, gold, silber, bronze) VALUES (?, 'm', ?, ?, ?, ?)", id, age_bew.alter, age_bew.gold, age_bew.silber, age_bew.bronze).execute(db).await.is_err() {
                return Err(ManageError::Internal{ message: "Error while inserting m alterklassen".to_string() });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::manage::schema::KategorieVorlage;
    use super::*;
    #[test]
    pub fn test_vorlagen() {
        check_vorlagen("vorlagen/".to_string()).unwrap();
    }

    #[test]
    pub fn get_vorlage_2023_1() {
        let _ = get_kat_from_vorlage("vorlagen/".to_string(), 2023, KategorieVorlage { bjs: Some(true), dosb: Some(true) , id: 1, changes: None });
    }

    #[sqlx::test]
    pub async fn create_gym_test_event() {
        use std::time::Instant;
        let now = Instant::now();

        let _ = fs::remove_file("testData/Test_Event_2024.db");
        let input_file = "testData/gym_event.json";
        let reader = File::open(input_file).unwrap();

        let event: EventConstructor = serde_json::from_reader(reader).unwrap();

        create_event("testData/".to_string(), "vorlagen/".to_string(), event).await.unwrap();

        println!("Created Event: {:.2?}", now.elapsed());
    }
    
    #[test]
    pub fn get_list_of_events() {
        let k_r = get_kat_list_from_vorlage("vorlagen/".to_string(), 2023);
        let k = k_r.unwrap();
        println!("{:?}", k);
    }
}