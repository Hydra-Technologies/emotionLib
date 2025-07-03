pub mod schema;

use crate::manage::schema::{BjsAlterBewertung, EventConstructor, Kategorie};
use log::{debug, info, warn};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::string::String;
use std::ffi::OsStr;
use walkdir;

use self::schema::{BjsKategorieConstructor, DosbAlterBewertung, DosbKategorieConstructor};

#[derive(Debug)]
pub enum ManageError {
    Internal { message: String },
    Conflict { message: String },
    NotFound { message: String },
    BadReque { message: String },
}

pub async fn create_event(
    school_dir: String,
    vorlagen_dir: String,
    id: String,
    data: schema::EventConstructor,
) -> Result<(), ManageError> {
    let db_url = format!("{}{}.db", school_dir, id);
    // check if database exists
    if Sqlite::database_exists(db_url.as_str())
        .await
        .unwrap_or(true)
    {
        return Err(ManageError::Conflict {
            message: "Event Alread exists (or error)".to_string(),
        });
    }
    info!("checked if db exists ");

    // create Database
    match Sqlite::create_database(db_url.as_str()).await {
        Err(e) => {
            info!("{}", e.to_string());
            return Err(ManageError::Internal {
                message: [
                    "Something went wrong while creating Table".to_string(),
                    e.to_string(),
                ]
                .join(""),
            });
        }
        Ok(_) => (),
    };
    info!("created DB");

    let con = SqlitePool::connect(db_url.as_str()).await.unwrap();

    // write the database schema to the file
    sqlx::migrate!("./migrations").run(&con).await.unwrap();

    info!("migrated DB");
    if data.bjs_bewertung.is_some() {
        insert_bjs_bewertungen(&con, data.bjs_bewertung.unwrap()).await?;
    } else {
        let path = format!("{}{}/init.json", vorlagen_dir, data.vorlage);

        let reader = std::io::BufReader::new(File::open(path).unwrap());
        let bjs: EventConstructor = serde_json::from_reader(reader).unwrap();

        insert_bjs_bewertungen(&con, bjs.bjs_bewertung.unwrap()).await?;
    }

    if data.kategorien.is_some() {
        for kat in data.kategorien.unwrap() {
            let _ = match kat {
                schema::ConstructKategorie::Kategorie(k) => insert_kat_in_db(&con, k).await,
                schema::ConstructKategorie::Vorlage(v) => {
                    insert_kat_in_db(
                        &con,
                        get_kat_from_vorlage(vorlagen_dir.clone(), data.vorlage, v)?,
                    )
                    .await
                }
            };
        }
    }

    return Ok(());
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
) -> Result<Vec<schema::OutsideKategorie>, ManageError> {
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
    vorlage: schema::KategorieVorlage,
) -> Result<schema::Kategorie, ManageError> {
    let path_string = format!("{}{}/{}.json", vorlagen_dir, year, vorlage.id);
    /*let path_string = [
        vorlagen_dir,
        year.to_string(),
        "/".to_string(),
        vorlage.id.to_string(),
        ".json".to_string(),
    ]
    .join("");*/

    let path = Path::new(&path_string);
    let reader = match File::open(path) {
        Err(_) => {
            let message = format!(
                "Kategorie {} konnte nicht gefunden werden",
                path.to_str().unwrap()
            );
            return Err(ManageError::NotFound { message });
        }
        Ok(f) => std::io::BufReader::new(f),
    };

    let kat: Kategorie = match serde_json::from_reader(reader) {
        Err(_) => {
            return Err(ManageError::Internal {
                message: "Error while reading Kategorie".to_string(),
            })
        }
        Ok(r) => r,
    };

    return Ok(if vorlage.changes.is_some() {
        let changes = vorlage.changes.unwrap();
        schema::Kategorie {
            name: changes.name.unwrap_or(kat.name),
            einheit: kat.einheit,
            kat_groupBJS: kat.kat_groupBJS,
            kat_groupDOSB: kat.kat_groupDOSB,
            digits_before: changes.digits_before.unwrap_or(kat.digits_before),
            digits_after: changes.digits_after.unwrap_or(kat.digits_after),
            versuche: changes.versuche.unwrap_or(kat.versuche),
            bjs: if changes.bjs.is_some() {
                let bjs_change = changes.bjs.unwrap();
                if kat.bjs.is_some() {
                    let bjs_kat = kat.bjs.unwrap();
                    Some(BjsKategorieConstructor {
                        a_m: bjs_kat.a_m,
                        a_w: bjs_kat.a_w,
                        c_m: bjs_kat.c_m,
                        c_w: bjs_kat.c_w,
                        formel: bjs_kat.formel,
                        altersklassen_m: bjs_change.altersklassen_m,
                        altersklassen_w: bjs_change.altersklassen_w,
                    })
                } else {
                    Some(BjsKategorieConstructor {
                        a_m: 0.0,
                        a_w: 0.0,
                        c_m: 0.0,
                        c_w: 0.0,
                        formel: "".to_string(),
                        altersklassen_m: bjs_change.altersklassen_m,
                        altersklassen_w: bjs_change.altersklassen_w,
                    })
                }
            } else {
                kat.bjs
            },

            dosb: if let Some(change_dosb) = changes.dosb {
                if let Some(kat_dosb) = kat.dosb {
                    Some(DosbKategorieConstructor {
                        altersklassen_m: change_dosb
                            .altersklassen_m
                            .into_iter()
                            .map(|a| {
                                if let Some(age_group) =
                                    kat_dosb.altersklassen_m.iter().find(|ka| ka.alter == a)
                                {
                                    DosbAlterBewertung {
                                        alter: a,
                                        bronze: age_group.bronze,
                                        silber: age_group.silber,
                                        gold: age_group.gold,
                                    }
                                } else {
                                    DosbAlterBewertung {
                                        alter: a,
                                        bronze: 0.0,
                                        silber: 0.0,
                                        gold: 0.0,
                                    }
                                }
                            })
                            .collect(),
                        altersklassen_w: change_dosb
                            .altersklassen_w
                            .into_iter()
                            .map(|a| {
                                if let Some(age_group) =
                                    kat_dosb.altersklassen_w.iter().find(|ka| ka.alter == a)
                                {
                                    DosbAlterBewertung {
                                        alter: a,
                                        bronze: age_group.bronze,
                                        silber: age_group.silber,
                                        gold: age_group.gold,
                                    }
                                } else {
                                    DosbAlterBewertung {
                                        alter: a,
                                        bronze: 0.0,
                                        silber: 0.0,
                                        gold: 0.0,
                                    }
                                }
                            })
                            .collect(),
                    })
                } else {
                    Some(DosbKategorieConstructor {
                        altersklassen_m: change_dosb
                            .altersklassen_m
                            .into_iter()
                            .map(|a| DosbAlterBewertung {
                                alter: a,
                                bronze: 0.0,
                                silber: 0.0,
                                gold: 0.0,
                            })
                            .collect(),
                        altersklassen_w: change_dosb
                            .altersklassen_w
                            .into_iter()
                            .map(|a| DosbAlterBewertung {
                                alter: a,
                                bronze: 0.0,
                                silber: 0.0,
                                gold: 0.0,
                            })
                            .collect(),
                    })
                }
            } else if vorlage.dosb.unwrap_or(false) {
                kat.dosb
            } else {
                None
            },
        }
    } else {
        schema::Kategorie {
            name: kat.name,
            einheit: kat.einheit,
            kat_groupBJS: kat.kat_groupBJS,
            kat_groupDOSB: kat.kat_groupDOSB,
            digits_before: kat.digits_before,
            digits_after: kat.digits_after,
            versuche: kat.versuche,
            bjs: if vorlage.bjs.unwrap_or(false) {
                kat.bjs
            } else {
                None
            },
            dosb: if vorlage.dosb.unwrap_or(false) {
                kat.dosb
            } else {
                None
            },
        }
    });
}

pub fn get_kat_by_vorlage(
    vorlagen_path: String,
    vorlage: i64,
) -> Result<Vec<schema::Kategorie>, ManageError> {
    let files = match fs::read_dir([vorlagen_path, vorlage.to_string(), "/".to_string()].join("")) {
        Ok(p) => p,
        Err(_) => {
            return Err(ManageError::Internal {
                message: "Vorlagen Dir not found".to_string(),
            })
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
                                return Err(ManageError::Internal {
                                    message: "Id nicht gefunden".to_string(),
                                })
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

async fn insert_bjs_bewertungen(
    db: &SqlitePool,
    bjs_alter_bewertungen: Vec<BjsAlterBewertung>,
) -> Result<(), ManageError> {
    for bew in bjs_alter_bewertungen {
        let gesch_str = bew.gesch.to_string();
        if sqlx::query!(
            "INSERT INTO ageGroups(age,gesch, gold, silber) VALUES (?,?,?,?)",
            bew.alter,
            gesch_str,
            bew.ehren,
            bew.sieger
        )
        .execute(db)
        .await
        .is_err()
        {
            return Err(ManageError::Internal {
                message: "Error while inserting bjs bewertungen".to_string(),
            });
        }
    }
    Ok(())
}

async fn insert_kat_in_db(db: &SqlitePool, kat: schema::Kategorie) -> Result<(), ManageError> {
    let lauf = kat.kat_groupBJS == 1 || kat.kat_groupBJS == 4;
    let id = match sqlx::query!("INSERT INTO kategorien(name, einheit, lauf, maxVers, digits_before, digits_after, kateGroupIdBJS, kateGroupIdDOSB) VALUES (?,?,?,?,?,?,?,?)",
        kat.name, kat.einheit, lauf, kat.versuche, kat.digits_before, kat.digits_after, kat.kat_groupBJS, kat.kat_groupDOSB).execute(db).await {
        Ok(r) => r.last_insert_rowid(),
        Err(_e) => return Err(ManageError::Internal{ message: "Error while inserting into Kategorien".to_string() })
    };

    info!("Inserted basic");

    // inserting BJS
    if kat.bjs.is_some() {
        let bjs = kat.bjs.unwrap();

        // the a and c numbers
        if sqlx::query!(
            "INSERT INTO formVars(katId, gesch, a, c) VALUES (?,'w',?,?)",
            id,
            bjs.a_w,
            bjs.c_w
        )
        .execute(db)
        .await
        .is_err()
        {
            return Err(ManageError::Internal {
                message: "Error while inserting w a and c".to_string(),
            });
        }
        if sqlx::query!(
            "INSERT INTO formVars(katId, gesch, a, c) VALUES (?,'m',?,?)",
            id,
            bjs.a_m,
            bjs.c_m
        )
        .execute(db)
        .await
        .is_err()
        {
            return Err(ManageError::Internal {
                message: "Error while inserting m a and c".to_string(),
            });
        }

        // the age groups
        for age in bjs.altersklassen_w {
            match sqlx::query!(
                "INSERT INTO bjsKat(katId, gesch, age) VALUES (?, 'w', ?)",
                id,
                age
            )
            .execute(db)
            .await
            {
                Err(e) => {
                    info!("{}", e);
                    return Err(ManageError::Internal {
                        message: "Error while inserting w alterklassen ".to_string(),
                    });
                }
                Ok(_) => (),
            }
        }

        for age in bjs.altersklassen_m {
            if sqlx::query!(
                "INSERT INTO bjsKat(katId, gesch, age) VALUES (?, 'm', ?)",
                id,
                age
            )
            .execute(db)
            .await
            .is_err()
            {
                return Err(ManageError::Internal {
                    message: "Error while inserting m alterklassen".to_string(),
                });
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
    use super::*;
    use crate::manage::schema::KategorieVorlage;
    use std::fs;
    #[test]
    pub fn test_vorlagen() {
        check_vorlagen("vorlagen/".to_string()).unwrap();
    }

    #[test]
    pub fn get_vorlage_2023_1() {
        let _ = get_kat_from_vorlage(
            "vorlagen/".to_string(),
            2023,
            KategorieVorlage {
                bjs: Some(true),
                dosb: Some(true),
                id: 1,
                changes: None,
            },
        );
    }

    #[sqlx::test]
    pub async fn create_gym_test_event() {
        use std::time::Instant;
        let now = Instant::now();

        let _ = fs::remove_file("testData/Test_Event_2024.db");
        let input_file = "testData/gym_event.json";
        let reader = File::open(input_file).unwrap();

        let event: EventConstructor = serde_json::from_reader(reader).unwrap();

        create_event("testData/".to_string(), "vorlagen/".to_string(), event)
            .await
            .unwrap();

        println!("Created Event: {:.2?}", now.elapsed());
    }

    #[test]
    pub fn get_list_of_events() {
        let k_r = get_kat_list_from_vorlage("vorlagen/".to_string(), 2023);
        let k = k_r.unwrap();
        println!("{:?}", k);
    }
}
