pub mod search_schema;
use futures::future::join_all;
use sha256::TrySha256Digest;
use sqlx::SqlitePool;
use std::{path::Path};
use actix_web::HttpResponse;
use crate::{InternalServerf, NotFoundf};
use crate::dosb_eval::DOSBEvaluator;
use crate::bjs_eval::BJSEvaluator;
use crate::model::Attempt;

#[derive(Debug)]
pub struct SchuelerResultConstructor {
    id: i64,
    bjs_punkte: Vec<i64>,
    dosb_punkte: i32,
    kat_groups_bjs: Vec<i64>,
    kat_groups_dosb: Vec<i64>,
}

pub async fn search_database(
    db: &SqlitePool,
    dosb_db: &SqlitePool,
    bjs_db: &SqlitePool
) -> Result<Vec<search_schema::SchuelerResult>, HttpResponse> {

    // get all schueler from the database
    let all_schueler = match sqlx::query!("SELECT id, age, gesch FROM schueler;").fetch_all(db).await {
        Ok(r) => r,
        Err(e) => return Err(InternalServerf!("There was an Error getting the schueler from the database {}", e))
    };

    let mut schueler_data = vec![];
    for schueler in all_schueler {
        // get all attempts of the student
        let id = schueler.id.unwrap();
        let age = schueler.age.unwrap();
        let gender= schueler.gesch.unwrap().chars().nth(0).unwrap();
        let attempts_rec = match sqlx::query!("SELECT kategorieId as category, wert as result FROM versuch WHERE schuelerId = ?", id).fetch_all(db).await {
            Ok(r) => r,
            Err(e) => return Err(InternalServerf!("There was an Error getting the schueler attempts from the database {} for schueler {}", e, id))
        };

        let attempts: Vec<Attempt> = attempts_rec.into_iter().map(|a| Attempt {
            category: a.category,
            result: a.result
        }).collect();

        // now we calculate the medals
        let dosb_evaluator = DOSBEvaluator {
            db: dosb_db
        };

        let bjs_evaluator= BJSEvaluator {
            db: bjs_db 
        };
        schueler_data.push(
            search_schema::SchuelerResult {
                id,
                bjs_punkte: bjs_evaluator.calculate_points_sum(age, gender, attempts.clone()).await?,
                bjs_urkunde: bjs_evaluator.get_medal(age,gender,attempts.clone()).await?,
                dosb_punkte: dosb_evaluator.calculate_points(age, gender, attempts.clone()).await? as i64,
                dosb_abzeichen: dosb_evaluator.get_medal(age,gender,attempts).await?
            }
        )
    }

    return Ok(schueler_data);
}

pub async fn search_database_extesive(
    db: &SqlitePool,
    dosb_db: &SqlitePool,
    bjs_db: &SqlitePool
) -> Result<Vec<search_schema::SchuelerResultExtensive>, HttpResponse> {
    let schueler_data = search_database(db, dosb_db, bjs_db).await?;
    let data_result = join_all(
        schueler_data
            .into_iter()
            .map(|r| async { result2extensive(r, &db, dosb_db, bjs_db).await })
    ) .await;
    
    let mut data_ext = vec![];
    for d in data_result{
        data_ext.push(d?);
    }
    return Ok(data_ext);

}

pub async fn get_db_hash(db_path: String) -> String {
    return Path::new(&db_path).async_digest().await.unwrap();
}

pub async fn result2extensive(
    result: search_schema::SchuelerResult,
    db: &SqlitePool,
    dosb_db: &SqlitePool,
    bjs_db: &SqlitePool
) -> Result<search_schema::SchuelerResultExtensive, HttpResponse> {
    let schueler = match sqlx::query!("SELECT age, gesch FROM schueler WHERE id = ?;", result.id).fetch_one(db).await {
        Ok(r) => r,
        Err(sqlx::Error::RowNotFound) => return Err(NotFoundf!("The schueler {} was not found in the database", result.id)),
        Err(e) => return Err(InternalServerf!("There was an Error getting the schueler with the id {} from the database {}", result.id, e))
    };
    let age = schueler.age.unwrap();
    let gender= schueler.gesch.unwrap().chars().nth(0).unwrap();

    let attempts_rec = match sqlx::query!("SELECT kategorieId as category, wert as result FROM versuch WHERE schuelerId = ?", result.id).fetch_all(db).await {
        Ok(r) => r,
        Err(e) => return Err(InternalServerf!("There was an Error getting the schueler attempts from the database {} for schueler {}", e, result.id))
    };

    let attempts: Vec<Attempt> = attempts_rec.into_iter().map(|a| Attempt {
        category: a.category,
        result: a.result
    }).collect();


    // get the top results of dosb
    let dosb_evaluator = DOSBEvaluator {
        db: dosb_db
    };
    let top_dosb = dosb_evaluator.get_top_attempts(age, gender,attempts.clone()).await?;

    // get the top results of bjs
    let bjs_evaluator= BJSEvaluator {
        db: bjs_db 
    };
    let top_bjs= bjs_evaluator.get_top_attempts(age, gender,attempts.clone()).await?;

    // now we join them without creating duplicates
    let mut single_results = top_dosb;
    for a in top_bjs {
        if !single_results.contains(&a) {
            single_results.push(a);
        }
    }


    return Ok(search_schema::SchuelerResultExtensive {
        id: result.id,
        bjs_punkte: result.bjs_punkte,
        bjs_urkunde: result.bjs_urkunde,
        dosb_punkte: result.dosb_punkte,
        dosb_abzeichen: result.dosb_abzeichen,
        single_results
    });
}

#[cfg(test)]
mod tests {
}
