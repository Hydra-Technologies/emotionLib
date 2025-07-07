use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
pub use emotion_lib_derive as macros;

// pub mod manage;
pub mod search;
pub mod auth;
mod model;
 pub mod schema;
#[macro_use]
pub mod http_res;
pub mod dosb_eval; pub mod bjs_eval;

#[derive(Serialize, Deserialize)]
pub struct UploadSchuelerResult {
    pub valid: Vec<schema::UploadSchueler>,
    pub age_invalid: Vec<schema::UploadSchueler>,
    pub gesch_invalid: Vec<schema::UploadSchueler>,
    pub id_invalid: Vec<schema::UploadSchueler>,
    pub id_conflict: Vec<schema::UploadSchueler>,
}

pub mod interact {
    use crate::dosb_eval::DOSBEvaluator;
    use crate::bjs_eval::BJSEvaluator;
    use crate::model;
    use crate::schema;
    use crate::model::Attempt;
    use crate::UploadSchuelerResult;
    use crate::search::search_schema;
    use crate::search::result2extensive;

    use regex::Regex;
    use sqlx::SqlitePool;
    use std::string::String;
    use std::time::{SystemTime, UNIX_EPOCH};
    use actix_web::HttpResponse;

    async fn get_attempts(id: i64, db: &SqlitePool) -> Result<Vec<Attempt>,HttpResponse>{
        // get all attempts of the student
        let attempts_rec = match sqlx::query!("SELECT kategorieId as category, wert as result FROM versuch WHERE schuelerId = ?", id).fetch_all(db).await {
            Ok(r) => r,
            Err(e) => return Err(InternalServerf!("There was an Error getting the schueler attempts from the database {} for schueler {}", e, id))
        };

        let attempts: Vec<Attempt> = attempts_rec.into_iter().map(|a| Attempt {
            category: a.category,
            result: a.result
        }).collect();

        return Ok(attempts);
    }
    async fn get_schueler_data(id: i64, db: &SqlitePool) -> Result<(i64, char), HttpResponse> {
        let schueler = match sqlx::query!("SELECT age, gesch FROM schueler WHERE id = ?" ,id).fetch_one(db).await {
            Ok(r) => r,
            Err(sqlx::Error::RowNotFound) => return Err(NotFoundf!("The Student {} was not found in the Database", id)),
            Err(e) => return Err(InternalServerf!("There was an Error gettin the student from the database: {}", e))
        };

        Ok((schueler.age.unwrap(), schueler.gesch.unwrap().chars().nth(0).unwrap()))
    }

    pub async fn get_schueler(
        id: &i32,
        db: &SqlitePool,
        dosb_db: &SqlitePool,
        bjs_db: &SqlitePool,
    ) -> Result<search_schema::SchuelerResultExtensive, HttpResponse> {
        let attempts = get_attempts(id.clone() as i64, db).await?;
        let (age, gender) = get_schueler_data(id.clone() as i64, db).await?;
        // now we calculate the medals
        let dosb_evaluator = DOSBEvaluator {
            db: dosb_db
        };

        let bjs_evaluator= BJSEvaluator {
            db: bjs_db 
        };

        Ok(result2extensive(search_schema::SchuelerResult {
                id: id.clone() as i64,
                bjs_punkte: bjs_evaluator.calculate_points_sum(age, gender, attempts.clone()).await?,
                bjs_urkunde: bjs_evaluator.get_medal(age,gender,attempts.clone()).await?,
                dosb_punkte: dosb_evaluator.calculate_points(age, gender, attempts.clone()).await? as i64,
                dosb_abzeichen: dosb_evaluator.get_medal(age,gender,attempts).await?
            },
            db,
            dosb_db,
            bjs_db
        ).await?)

    }

    pub async fn get_dosb_task_for_schueler(
        id: i32,
        db: &SqlitePool,
        dosb_db: &SqlitePool
    ) -> Result<Vec<Vec<i64>>, HttpResponse> {
        let attempts = get_attempts(id.clone() as i64, db).await?;
        let (age, gender) = get_schueler_data(id.clone() as i64, db).await?;
        let dosb_evaluator = DOSBEvaluator {
            db: dosb_db
        };
        return dosb_evaluator.get_missing_categorys(age, gender, attempts.iter().map(|a| a.category).collect()).await
    }

    pub async fn get_bjs_task_for_schueler(
        id: i32,
        db: &SqlitePool,
        bjs_db: &SqlitePool,
    ) -> Result<Vec<Vec<i64>>, HttpResponse> {
        let attempts = get_attempts(id.clone() as i64, db).await?;
        let (age, gender) = get_schueler_data(id.clone() as i64, db).await?;
        let bjs_evaluator = DOSBEvaluator {
            db: bjs_db 
        };
        return bjs_evaluator.get_missing_categorys(age, gender, attempts.iter().map(|a| a.category).collect()).await
    }

    pub async fn upload_schueler(
        schueler_list: Vec<schema::UploadSchueler>,
        db: &SqlitePool,
    ) -> UploadSchuelerResult {
        let mut result = UploadSchuelerResult {
            valid: vec![],
            age_invalid: vec![],
            gesch_invalid: vec![],
            id_invalid: vec![],
            id_conflict: vec![],
        };

        // check if the age is resonable

        for schueler in schueler_list.into_iter() {
            let age: i8;
            if schueler.age.is_some() && schueler.age.clone().unwrap() != -1 {
                age = schueler.age.clone().unwrap();
                if !(5..25).contains(&age) {
                    result.age_invalid.push(schueler);
                    continue;
                }
            } else if schueler.bday.is_some() && schueler.bday.clone().unwrap() != "-1" {
                let b_day_str = schueler.bday.clone().unwrap();
                let now = (SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    / 31536000) as u64
                    + 1970;
                let re = Regex::new("(20|19)[0-9][0-9]").unwrap();
                let b_day = match re.find(b_day_str.as_str()) {
                    Some(b) => b.as_str().parse::<u64>().unwrap(),
                    None => {
                        result.age_invalid.push(schueler);
                        continue;
                    }
                };
                age = (now - b_day) as i8;
                if !(5..25).contains(&age) {
                    result.age_invalid.push(schueler);
                    continue;
                }
            } else {
                result.age_invalid.push(schueler);
                continue;
            }

            if !(1000..9999).contains(&schueler.id) {
                result.id_invalid.push(schueler);
                continue;
            }

            if !['m', 'w'].contains(&schueler.gesch) {
                result.gesch_invalid.push(schueler);
                continue;
            }

            let id = schueler.id.clone();
            let gesch = schueler.gesch.clone().to_string();

            match sqlx::query!(
                "INSERT INTO schueler(id, gesch, age) VALUES (?,?,?)",
                id,
                gesch,
                age,
            )
            .execute(db)
            .await
            {
                Ok(_) => result.valid.push(schueler),
                Err(_) => result.id_conflict.push(schueler),
            }
        }
        return result;
    }
    pub async fn get_all_versuch_for_kat(
        id: i32,
        kat_id: i32,
        db: &SqlitePool,
    ) -> Result<Vec<model::NormVersuch>, HttpResponse> {
        return match sqlx::query_as!(model::NormVersuch, "SELECT id, schuelerId as schueler_id, kategorieId as kategorie_id, wert, isReal as is_real, mTime as ts_recording FROM versuch WHERE schuelerId = ? AND kategorieId = ?", id, kat_id).fetch_all(db).await {
            Ok(r) => Ok(r),
            Err(e) => Err(InternalServerf!("There was an error with the query {}",e))
        };
    }

    pub async fn get_top_versuch_by_kat(
        id: i32,
        kat_id: i32,
        db: &SqlitePool,
        dosb_db: &SqlitePool,
        bjs_db: &SqlitePool,
    ) -> Result<model::NormVersuch, HttpResponse> {
        // get all attempts of the student
        let attempts_rec = match sqlx::query!("SELECT kategorieId as category, wert as result FROM versuch WHERE schuelerId = ? AND kategorieId = ?", id, kat_id).fetch_all(db).await {
            Ok(r) => r,
            Err(e) => return Err(InternalServerf!("There was an Error getting the schueler attempts from the database {} for schueler {}", e, id))
        };

        let attempts: Vec<Attempt> = attempts_rec.into_iter().map(|a| Attempt {
            category: a.category,
            result: a.result
        }).collect();

        let (age, gender) = get_schueler_data(id.clone() as i64, db).await?;

        // get the top results of dosb
        let dosb_evaluator = DOSBEvaluator {
            db: dosb_db
        };
        let top_dosb = dosb_evaluator.get_top_attempts(age, gender,attempts.clone()).await?;

        // get the top results of bjs
        let bjs_evaluator = BJSEvaluator {
            db: bjs_db 
        };
        let top_bjs = bjs_evaluator.get_top_attempts(age, gender,attempts.clone()).await?;

        // now we join them without creating duplicates
        if top_dosb.len() == 0 && top_dosb.len() == 0 {
            return Err(Conflictf!("The category {} is not required for students with this age and gender", kat_id));
        }

        let att = if top_dosb.len() > 0 {
            top_dosb[0]
        } else {
            top_bjs[0]
        };

        // becaus i didnt carry the id of thes attempts through all of this I need to map them to
        // the corresponding versuch id in the database.
        let v = match sqlx::query_as!(model::NormVersuch, r#"
        SELECT id, schuelerId as schueler_id, kategorieId as kategorie_id, wert, isReal as is_real, mTime as ts_recording FROM versuch 
        WHERE schuelerId = ? AND kategorieId = ? AND wert = ?"#, 
        id, att.category, att.result).fetch_one(db).await {
            Ok(r) => r,
            Err(e) => return Err(InternalServerf!("Error while rematching the attemts {}" ,e))
        };
        return Ok(v);
    }

    pub async fn get_top_versuch_in_bjs(
        id: i32,
        db: &SqlitePool,
        bjs_db: &SqlitePool,
    ) -> Result<Vec<schema::NormVersuchBJS>, HttpResponse> {
        let attempts = get_attempts(id.clone() as i64, db).await?;
        let (age, gender) = get_schueler_data(id.clone() as i64, db).await?;

        let bjs_evaluator= BJSEvaluator {
            db: bjs_db 
        };

        let top_bjs_attemtps = bjs_evaluator.get_top_attempts(age, gender, attempts).await?;

        let mut top_bjs_norm = vec![];
        for att in top_bjs_attemtps {
            let v = match sqlx::query_as!(model::NormVersuch, r#"
            SELECT id, schuelerId as schueler_id, kategorieId as kategorie_id, wert, isReal as is_real, mTime as ts_recording FROM versuch 
            WHERE schuelerId = ? AND kategorieId = ? AND wert = ?"#, 
            id, att.category, att.result).fetch_one(db).await {
                Ok(r) => r,
                Err(e) => return Err(InternalServerf!("Error while rematching the attemts {}" ,e))
            };

            top_bjs_norm.push(v);
        }

        let mut top_bjs_result = vec![];
        for att in top_bjs_norm {
            top_bjs_result.push(schema::NormVersuchBJS {
                id: att.id,
                schueler_id: att.schueler_id,
                kategorie_id: att.kategorie_id,
                wert: att.wert,
                punkte: bjs_evaluator.calculate_points(gender, &Attempt { category:  att.kategorie_id, result: att.wert }).await?,
                ts_recording: att.ts_recording,
                is_real: att.is_real
            });
        }

        return Ok(top_bjs_result);
    }

    pub async fn get_top_versuch_in_dosb(
        id: i32,
        db: &SqlitePool,
        dosb_db: &SqlitePool,
    ) -> Result<Vec<schema::NormVersuchDosb>, HttpResponse> {
        let attempts = get_attempts(id.clone() as i64, db).await?;
        let (age, gender) = get_schueler_data(id.clone() as i64, db).await?;

        let dosb_evaluator= DOSBEvaluator {
            db: dosb_db 
        };

        let top_dosb_attemtps = dosb_evaluator.get_top_attempts(age, gender, attempts).await?;

        let mut top_dosb_norm = vec![];
        for att in top_dosb_attemtps {
            let v = match sqlx::query_as!(model::NormVersuch, r#"
            SELECT id, schuelerId as schueler_id, kategorieId as kategorie_id, wert, isReal as is_real, mTime as ts_recording FROM versuch 
            WHERE schuelerId = ? AND kategorieId = ? AND wert = ?"#, 
            id, att.category, att.result).fetch_one(db).await {
                Ok(r) => r,
                Err(e) => return Err(InternalServerf!("Error while rematching the attemts {}" ,e))
            };

            top_dosb_norm.push(v);
        }
        
        let mut top_dosb_result = vec![];
        for att in top_dosb_norm {
            top_dosb_result.push(schema::NormVersuchDosb {
                id: att.id,
                schueler_id: att.schueler_id,
                kategorie_id: att.kategorie_id,
                wert: att.wert,
                dosb: dosb_evaluator.get_medal_for_attempt(age, gender, &Attempt { category:  att.kategorie_id, result: att.wert }).await?,
                ts_recording: att.ts_recording,
                is_real: att.is_real
            })
        }
        return Ok(top_dosb_result);
    }

    pub async fn get_bjs_points(id: i32, db: &SqlitePool, bjs_db: &SqlitePool) -> Result<i32, HttpResponse> {
        let attempts = get_attempts(id.clone() as i64, db).await?;
        let (age, gender) = get_schueler_data(id.clone() as i64, db).await?;

        // get the top results of bjs
        let bjs_evaluator= BJSEvaluator {
            db: bjs_db 
        };
        return Ok(bjs_evaluator.calculate_points_sum(age, gender, attempts).await? as i32);
    }

    pub async fn needs_kat(
        schueler_id: i32,
        kategorie_id: i32,
        db: &SqlitePool,
        dosb_db: &SqlitePool,
        bjs_db: &SqlitePool
    ) -> Result<schema::NeedsKat, HttpResponse> {
        let (age, gender) = get_schueler_data(schueler_id.clone() as i64, db).await?;

        let bjs_evaluator= BJSEvaluator {
            db: bjs_db 
        };
        let dosb_evaluator= DOSBEvaluator {
            db: dosb_db 
        };

        let needed_dosb: Vec<i32> = dosb_evaluator.get_needed_categorys(age,gender).await?.iter().map(|k| k.id as i32).collect();
        let needed_bjs: Vec<i32> = bjs_evaluator.get_needed_categorys(age,gender).await?.iter().map(|k| k.id as i32).collect();
        return Ok(schema::NeedsKat {
            dosb: needed_dosb.contains(&kategorie_id),
            bjs: needed_bjs.contains(&kategorie_id)
        });
    }

    pub async fn add_versuch(
        versuch: schema::SimpleVersuch,
        vouch_name: String,
        db: &SqlitePool,
    ) -> Result<i32, i32> {
        if !check_schueler_id(&versuch.schueler_id) {
            return Err(400);
        }
        if !check_kategorie_id(&versuch.kategorie_id, db).await {
            return Err(400);
        }

        // get Current time
        let current_timestamp: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let v = sqlx::query_as!(model::VersuchId, r#"
    INSERT INTO versuch(id, aufsichtId, schuelerId, kategorieId, wert, mTime, isReal) VALUES ((SELECT ifNUll(MAX(id)+1, 1) FROM versuch), ?, ?, ?, ?, ?, true) RETURNING id;
    "#, vouch_name, versuch.schueler_id, versuch.kategorie_id, versuch.wert, current_timestamp)
            .fetch_one(db).await.unwrap();

        return Ok(v.id.unwrap() as i32);
    }

    pub async fn set_is_real(id: i32, is_real: bool, db: &SqlitePool) -> bool {
        let r = sqlx::query("UPDATE versuch SET isReal = ? WHERE id = ?")
            .bind(is_real)
            .bind(id)
            .execute(db)
            .await;
        return match r {
            Err(_e) => false,
            Ok(q) => q.rows_affected() == 1,
        };
    }

    pub async fn get_all_kat(db: &SqlitePool) -> Vec<schema::SimpleKategorie> {
        let query_result = sqlx::query_as!(
            model::SimpleKategorie,
            r#"
        SELECT id, name FROM kategorien
        "#
        )
        .fetch_all(db)
        .await
        .unwrap();

        return query_result
            .into_iter()
            .map(|k| schema::SimpleKategorie {
                id: k.id.unwrap() as i32,
                name: k.name.unwrap(),
            })
            .collect();
    }

    pub async fn get_kategorie(id: i32, db: &SqlitePool) -> schema::Kategorie {
        let result = sqlx::query_as!(model::Kategorie, r#"
        SELECT id, name, einheit, maxVers as max_vers, digits_before, digits_after FROM kategorien WHERE id = ?
        "#, id).fetch_one(db).await;
        return kategorie_model2schema(result.unwrap());
    }


    async fn check_kategorie_id(id: &i32, db_con: &SqlitePool) -> bool {
        let query_response = sqlx::query(
            r#"
            SELECT id FROM kategorien WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(db_con)
        .await;

        return match query_response {
            Ok(_a) => true,
            Err(_e) => false,
        };
    }

    fn kategorie_model2schema(m: model::Kategorie) -> schema::Kategorie {
        schema::Kategorie {
            id: m.id.unwrap(),
            name: m.name.unwrap(),
            lauf: false,
            einheit: m
                .einheit
                .unwrap()
                .chars()
                .next()
                .expect("no Unit was given"),
            max_vers: m.max_vers.unwrap(),
            digits_before: m.digits_before.unwrap(),
            digits_after: m.digits_after.unwrap(),
            kat_group_id: 0,
        }
    }

    fn check_schueler_id(id: &i32) -> bool {
        return id < &9999 && id >= &1000;
    }
}

#[cfg(test)]
mod tests {
}
