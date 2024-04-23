pub mod search_schema;
use sqlx::SqlitePool;
use std::collections::HashMap;
use futures::future::join_all;
use self::search_schema::{SchuelerResult, SchuelerResultExtensive};

#[derive(Debug)]
pub enum SearchError {
    InternalError { message: String, error: String },
    NotFound { message: String },
    BadRequest { meassage: String }
}

#[derive(Debug)]
pub struct SchuelerResultConstructor {
    id: i64,
    bjs_punkte: i64,
    dosb_punkte: i32,
    kat_groups: Vec<i64>,
}


pub async fn search_database(db: &SqlitePool) -> Result<Vec<search_schema::SchuelerResult>, SearchError> {
    // first get all student Data from the database
    // here we get a list of the best trys of wich the medal is already calculated
    let all_student_data = match sqlx::query_file!("src/search/selectVersucheOfStudents.sql").fetch_all(db).await {
        Ok(r) => r,
        Err(e) => return Err(SearchError::InternalError { message: "There was an error getting the schueler from the Database".to_string(), error: e.to_string() })
    };

    
    // this combines the list of trys into a list of schueler
    let mut schueler_map: HashMap<i64, SchuelerResultConstructor> = HashMap::new();
    for versuch in all_student_data {
        match schueler_map.get(&versuch.schueler_id) {
            Some(r) => {
                let mut kat_groups: Vec<i64>  = r.kat_groups.clone();
                kat_groups.push(versuch.kat_group_id.unwrap());
                
                schueler_map.insert(versuch.schueler_id, SchuelerResultConstructor {
                    id: versuch.schueler_id,
                    bjs_punkte: versuch.bjs_punkte.unwrap_or(0) + r.bjs_punkte,
                    dosb_punkte: versuch.dosb_abzeichen + r.dosb_punkte,
                    kat_groups: kat_groups
                });
            },
            None => {
                
                println!("{:?}", versuch);
                schueler_map.insert(versuch.schueler_id, SchuelerResultConstructor {
                id: versuch.schueler_id,
                bjs_punkte: versuch.bjs_punkte.unwrap_or(0),
                dosb_punkte: versuch.dosb_abzeichen,
                kat_groups: vec![versuch.kat_group_id.unwrap()]
            }); }
        };
    }

    // now we need to calculate the medals
    let schueler_data: Vec<search_schema::SchuelerResult> =join_all(schueler_map.drain().map(|r| r.1).map(|c_par| async {
        let c = c_par;
        let user_medal_result = sqlx::query!("SELECT silber, gold FROM ageGroups INNER JOIN schueler ON ageGroups.age = schueler.age AND ageGroups.gesch = schueler.gesch WHERE schueler.id = ?", c.id).fetch_one(db).await;
        search_schema::SchuelerResult {
            id: c.id,
            bjs_punkte: c.bjs_punkte,
            bjs_urkunde: if let Ok(e) = user_medal_result { 
                if c.bjs_punkte<e.silber.unwrap() { search_schema::BJSUrkunde::Teilnehmer }
                else if c.bjs_punkte<e.gold.unwrap() { search_schema::BJSUrkunde::Sieger }
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

async fn result2extensive(result: search_schema::SchuelerResult, db: &SqlitePool) -> search_schema::SchuelerResultExtensive {
    let singel_results = match sqlx::query_file_as!(search_schema::SingleResult, "src/search/getBestTrys.sql", result.id).fetch_all(db).await {
        Ok(r) => r,
        Err(_)  => vec![]
    };

    return SchuelerResultExtensive {
        id: result.id,
        bjs_punkte: result.bjs_punkte,
        bjs_urkunde: result.bjs_urkunde,
        dosb_punkte: result.dosb_punkte,
        dosb_abzeichen: result.dosb_abzeichen,
        single_results: singel_results,
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use super::*;

    #[sqlx::test]
    async fn search() {
        let db = SqlitePool::connect("db/emotion1.db").await.unwrap();
        println!("Results: \n{:?}", search_database(&db).await);
    }
}