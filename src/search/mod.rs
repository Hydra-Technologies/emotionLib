pub mod search_schema;
use crate::schema::SimpleVersuch;
use futures::future::join_all;
use sha256::TrySha256Digest;
use sqlx::SqlitePool;
use std::{collections::HashMap, path::Path};

#[derive(Debug)]
pub enum SearchError {
    InternalError { message: String, error: String },
    NotFound { message: String },
    BadRequest { meassage: String },
}

#[derive(Debug)]
pub struct SchuelerResultConstructor {
    id: i64,
    bjs_punkte: Vec<i64>,
    dosb_punkte: i32,
    kat_groups: Vec<i64>,
}

pub async fn search_database(
    db: &SqlitePool,
) -> Result<Vec<search_schema::SchuelerResult>, SearchError> {
    // first get all student Data from the database
    // here we get a list of the best trys of wich the medal is already calculated
    let student_data = match sqlx::query_file!("src/search/selectVersucheOfStudents.sql")
        .fetch_all(db)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(SearchError::InternalError {
                message: "There was an error getting the schueler from the Database".to_string(),
                error: e.to_string(),
            })
        }
    };

    // this combines the list of trys into a list of schueler
    let mut schueler_map: HashMap<i64, SchuelerResultConstructor> = HashMap::new();
    for versuch in student_data {
        let versuch_bjs_punkte = calc_points(
            SimpleVersuch {
                schueler_id: versuch.schueler_id.clone() as i32,
                wert: versuch.wert.clone().unwrap() as f32,
                kategorie_id: versuch.kategorie_id.clone() as i32,
            },
            db,
        )
        .await;
        match schueler_map.get(&versuch.schueler_id) {
            Some(r) => {
                let mut kat_groups: Vec<i64> = r.kat_groups.clone();
                let mut bjs_punkte: Vec<i64> = r.bjs_punkte.clone();
                kat_groups.push(versuch.kat_group_id.unwrap());
                if versuch_bjs_punkte > 0 {
                    bjs_punkte.push(versuch_bjs_punkte as i64);
                }

                schueler_map.insert(
                    versuch.schueler_id,
                    SchuelerResultConstructor {
                        id: versuch.schueler_id,
                        bjs_punkte,
                        dosb_punkte: versuch.dosb_abzeichen + r.dosb_punkte,
                        kat_groups,
                    },
                );
            }
            None => {
                schueler_map.insert(
                    versuch.schueler_id,
                    SchuelerResultConstructor {
                        id: versuch.schueler_id,
                        bjs_punkte: if versuch_bjs_punkte > 0 {
                            vec![versuch_bjs_punkte as i64]
                        } else {
                            vec![]
                        },
                        dosb_punkte: versuch.dosb_abzeichen,
                        kat_groups: vec![versuch.kat_group_id.unwrap()],
                    },
                );
            }
        };
    }

    let all_schueler = sqlx::query!("SELECT id FROM schueler")
        .fetch_all(db)
        .await
        .unwrap();
    // now we need to calculate the medals
    let schueler_data: Vec<search_schema::SchuelerResult> = join_all(all_schueler
        .into_iter()
        .map(|i| i.id.unwrap())
        .map(|id| {
            schueler_map
                .remove(&id)
                .unwrap_or(SchuelerResultConstructor {
                    id,
                    bjs_punkte: vec![],
                    dosb_punkte: 0,
                    kat_groups: vec![],
                })
        }).map(|c| async move {
            // for schueler with no trys
            if c.kat_groups.is_empty() {
                return search_schema::SchuelerResult {
                    id: c.id,
                    bjs_punkte: 0,
                    bjs_urkunde: search_schema::BJSUrkunde::None,
                    dosb_punkte: 0,
                    dosb_abzeichen: search_schema::DOSBAbzeichen::None,
                }
            }

            let user_medal_result = sqlx::query!("SELECT silber, gold FROM ageGroups INNER JOIN schueler ON ageGroups.age = schueler.age AND ageGroups.gesch = schueler.gesch WHERE schueler.id = ?", c.id).fetch_one(db).await;

            let bjs_punkte = add_bjs_points(c.bjs_punkte);
            search_schema::SchuelerResult {
                id: c.id,
                bjs_punkte,
                bjs_urkunde: if let Ok(e) = user_medal_result {
                    if bjs_punkte<e.silber.unwrap() { search_schema::BJSUrkunde::Teilnehmer }
                    else if bjs_punkte<e.gold.unwrap() { search_schema::BJSUrkunde::Sieger }
                    else { search_schema::BJSUrkunde::Ehren }
                } else {
                    search_schema::BJSUrkunde::None
                },
                dosb_punkte: c.dosb_punkte as i64,
                dosb_abzeichen: if [1, 2, 3, 4].iter().all(|g| c.kat_groups.contains(g)) { search_schema::DOSBAbzeichen::None }
                    else if c.dosb_punkte < 8  { search_schema::DOSBAbzeichen::Bronze }
                    else if c.dosb_punkte < 11  { search_schema::DOSBAbzeichen::Silber }
                    else { search_schema::DOSBAbzeichen::Gold }
            }
        })).await;

    // with all that we can return the data
    return Ok(schueler_data);
}

/// Add the top 3 Scores together
fn add_bjs_points(points: Vec<i64>) -> i64 {
    if points.len() < 4 {
        return points.iter().sum();
    }
    let mut m_points = points.clone();
    m_points.sort();

    m_points = m_points[1..=3].to_vec();
    return m_points.into_iter().sum();
}

pub async fn search_database_extesive(
    db: &SqlitePool,
) -> Result<Vec<search_schema::SchuelerResultExtensive>, SearchError> {
    let schueler_data = search_database(db).await?;
    Ok(join_all(
        schueler_data
            .into_iter()
            .map(|r| async { result2extensive(r, &db).await }),
    )
    .await)
}

pub async fn get_db_hash(db_path: String) -> String {
    return Path::new(&db_path).async_digest().await.unwrap();
}

async fn result2extensive(
    result: search_schema::SchuelerResult,
    db: &SqlitePool,
) -> search_schema::SchuelerResultExtensive {
    let singel_results = match sqlx::query_file_as!(
        search_schema::SingleResult,
        "src/search/getBestTrys.sql",
        result.id
    )
    .fetch_all(db)
    .await
    {
        Ok(r) => r,
        Err(_) => vec![],
    };

    return search_schema::SchuelerResultExtensive {
        id: result.id,
        bjs_punkte: result.bjs_punkte,
        bjs_urkunde: result.bjs_urkunde,
        dosb_punkte: result.dosb_punkte,
        dosb_abzeichen: result.dosb_abzeichen,
        single_results: singel_results,
    };
}

// This is just a copy of function in lib.rs
// I want to change the stucture so this is not nessesary anymore
pub async fn calc_points(versuch: SimpleVersuch, db: &SqlitePool) -> i32 {
    // get kategorie for calc point
    let kat_result = sqlx::query!(
        r#"
            SELECT name, a, c, kateGroupId as group_id FROM schueler
                INNER JOIN formVars ON formVars.gesch = schueler.gesch
                INNER JOIN kategorien ON formVars.katId = kategorien.id
            WHERE kategorien.id = ? and schueler.id = ?
            "#,
        versuch.kategorie_id,
        versuch.schueler_id
    )
    .fetch_one(db)
    .await;

    let kat = match kat_result {
        Ok(k) => k,
        Err(_e) => return -404,
    };

    let a = kat.a.unwrap();
    let c = kat.c.unwrap();
    let name = kat.name.unwrap();

    let group_id = kat.group_id.unwrap();
    let points = if group_id == 1 || group_id == 4 {
        // get distance
        // TODO: Get your distance from somewhere else this sucks
        let name_vec: Vec<&str> = name.split("m").collect();
        let distance = match name_vec[0].to_string().parse::<i32>() {
            Ok(d) => d,
            Err(_e) => return -500,
        };

        // look up zuschlag
        let zuschlag: f32 = if distance < 301 {
            0.24
        } else if distance < 401 {
            0.14
        } else {
            0.0
        };
        (((distance as f32 / (versuch.wert + zuschlag)) - a as f32) / c as f32) as i32
    } else {
        ((versuch.wert.sqrt() - a as f32) / c as f32) as i32
    };
    if points < 0 {
        return 0;
    } else if points > 900 {
        return -406;
    }
    return points;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn search() {
        let db = SqlitePool::connect("db/emotion1.db").await.unwrap();
        let result = search_database_extesive(&db).await;
        println!("{:#?}", result);
    }
}
