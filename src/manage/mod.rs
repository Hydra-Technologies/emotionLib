mod schema;

use std::fs;
use std::fs::File;
use std::path::Path;
use sqlx::{Sqlite, SqlitePool};
use actix_web::HttpResponse;
use sqlx::migrate::MigrateDatabase;
use walkdir;
use std::string::String;
use crate::manage::schema::{BjsAlterBewertung, EventConstructor, Kategorie };
use log::{info, warn};

pub async fn create_event(school_dir: String, data: schema::EventConstructor) -> Result<(), HttpResponse> {
    let db_url = [school_dir, data.name.clone(), ".db".to_string()].join("");
    // check if database exists
    if Sqlite::database_exists(db_url.as_str()).await.unwrap_or(true) {
        return Err(HttpResponse::Conflict().json(serde_json::json!({"message": "Event Alread exists (or error)"})));
    }
    info!("checked if db exists ");

    // create Database
    match Sqlite::create_database(db_url.as_str()).await {
        Err(e) =>  {
            info!("{}",e.to_string());
            return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Something went wrong while creating Table"})));},
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
        let path = ["vorlagen/".to_string(), data.vorlage.unwrap().to_string(), "/init.json".to_string()].join("");
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
                        Some(y) => insert_kat_in_db(&con, get_kat_from_vorlage("vorlagen/".to_string(), y, v)?).await,
                        None => return Err(HttpResponse::BadRequest().json(serde_json::json!({"message": "No Vorlage specified"})))
                    }
                }
            }?
        }
    }

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

pub fn get_kat_from_vorlage(vorlagen_dir: String, year: i32, vorlage: schema::KategorieVorlage) -> Result<schema::Kategorie, HttpResponse> {
    let path_string = [vorlagen_dir, year.to_string(), "/".to_string(), vorlage.id.to_string(), ".json".to_string()].join("");

    let path = Path::new(&path_string);
    let reader = match File::open(path) {
        Err(_) => {
            let message = format!("Kategorie {} konnte nicht gefunden werden", path.to_str().unwrap());
            return Err(HttpResponse::NotFound().json(serde_json::json!({"message": message})));
        },
        Ok(f) => std::io::BufReader::new(f)
    };

    let kat: Kategorie =  match serde_json::from_reader(reader) {
        Err(_) => return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while reading Kategorie"}))),
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

pub fn get_kat_by_vorlage(vorlagen_path: String, vorlage: i64) -> Result<Vec<schema::Kategorie>, HttpResponse> {
    let files = match fs::read_dir([vorlagen_path, vorlage.to_string(), "/".to_string()].join("")) {
        Ok(p) => p,
        Err(e) => return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Vorlagen Dir not found"})))
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
                            Err(e) => return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Id nicht gefunden"})))
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


async fn insert_bjs_bewertungen(db: &SqlitePool, bjs_alter_bewertungen: Vec<BjsAlterBewertung>) -> Result<(), HttpResponse> {
    info!("Hello before :)");
    for bew in bjs_alter_bewertungen {
        let gesch_str = bew.gesch.to_string();
        if sqlx::query!("INSERT INTO ageGroups(age,gesch, gold, silber) VALUES (?,?,?,?)", bew.alter, gesch_str , bew.ehren, bew.sieger).execute(db).await.is_err() {
            return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while inserting bjs bewertungen"})));
        }
    }
    info!("hello :> ");
    Ok(())
}

async fn insert_kat_in_db(db: &SqlitePool, kat: schema::Kategorie) -> Result<(), HttpResponse>{
    let form = ["{",&kat.digits_before.to_string(),";", kat.einheit.as_str(), "}, {",kat.digits_after.to_string().as_str(), "; c", &kat.einheit.to_string().as_str(), "}", &kat.einheit.to_string()].map(|r| r.to_string()).join("");
    let lauf = kat.kat_group == 1 || kat.kat_group == 4;
    let id = match sqlx::query!("INSERT INTO kategorien(name, einheit, lauf, maxVers, messungsForm, kateGroupId) VALUES (?,?,?,100,?,?)",
        kat.name, kat.einheit, lauf, form, kat.kat_group).execute(db).await {
        Ok(r) => r.last_insert_rowid(),
        Err(e) => return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while inserting into Kategorien", "err": e.to_string()})))
    };

    info!("Inserted basic");

    // inserting BJS
    if kat.bjs.is_some() {
        let bjs = kat.bjs.unwrap();

        // the a and c numbers
        if sqlx::query!("INSERT INTO formVars(katId, gesch, a, c) VALUES (?,'w',?,?)", id, bjs.a_w, bjs.c_w).execute(db).await.is_err() {
            return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while inserting w a and c"})));
        }
        if sqlx::query!("INSERT INTO formVars(katId, gesch, a, c) VALUES (?,'m',?,?)", id, bjs.a_m, bjs.c_m).execute(db).await.is_err() {
            return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while inserting m a and c"})));
        }

        // the age groups
        for age in bjs.altersklassen_w {
            match sqlx::query!("INSERT INTO bjsKat(katId, gesch, age) VALUES (?, 'w', ?)", id, age).execute(db).await {
                Err(e) => {
                    info!("{}", e);
                    return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while inserting w alterklassen "})));
                },
                Ok(_) => ()
            }
        }


        for age in bjs.altersklassen_m {
            if sqlx::query!("INSERT INTO bjsKat(katId, gesch, age) VALUES (?, 'm', ?)", id, age).execute(db).await.is_err() {
                return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error whsile inserting m alterklassen"})));
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
                return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while inserting w alterklassen"})));
            }
        }

        for age_bew in dosb.altersklassen_m {
            if sqlx::query!("INSERT INTO dosbKat(katId, gesch, age, gold, silber, bronze) VALUES (?, 'm', ?, ?, ?, ?)", id, age_bew.alter, age_bew.gold, age_bew.silber, age_bew.bronze).execute(db).await.is_err() {
                return Err(HttpResponse::InternalServerError().json(serde_json::json!({"message": "Error while inserting m alterklassen"})));
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
        let _ = fs::remove_file("testData/Test_Event_2024.db");
        let input_file = "testData/gym_event.json";
        let reader = File::open(input_file).unwrap();

        let event: EventConstructor = serde_json::from_reader(reader).unwrap();

        create_event("testData/".to_string(), event).await.unwrap();
    }
}