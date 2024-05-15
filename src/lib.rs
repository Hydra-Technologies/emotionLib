use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
pub mod manage;
mod model;
pub mod schema;
pub mod search;

// not_now_TODO write class with all the Functions (but not today)
pub struct EmotionCon {
    pub database: SqlitePool,
}
impl EmotionCon {
    /*async fn get_schueler(self, id: &i32) -> Result<schema::SimpleSchueler, i32> {
        interact::get_schueler(id, &self.database).await
    }*/
}

#[derive(Serialize, Deserialize)]
pub struct UploadSchuelerResult {
    pub valid: Vec<schema::UploadSchueler>,
    pub age_invalid: Vec<schema::UploadSchueler>,
    pub gesch_invalid: Vec<schema::UploadSchueler>,
    pub id_invalid: Vec<schema::UploadSchueler>,
    pub id_conflict: Vec<schema::UploadSchueler>,
}

pub mod interact {
    use crate::model;
    use crate::schema;
    use crate::UploadSchuelerResult;
    use regex::Regex;
    use sqlx::SqlitePool;
    use std::string::String;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub async fn get_schueler(id: &i32, db: &SqlitePool) -> Result<schema::SimpleSchueler, i32> {
        if !check_schueler_id(&id) {
            return Err(400);
        }

        let query_response = sqlx::query_as!(
            model::SimpleSchueler,
            r#"
        SELECT id, fName as first_name, lName as last_name FROM schueler WHERE id = ?
            "#,
            id
        )
        .fetch_one(db)
        .await;

        match query_response {
            Ok(schueler) => {
                let id = match schueler.id {
                    Some(i) => i,
                    None => return Err(406),
                };
                let first_name = match schueler.first_name {
                    Some(f) => f,
                    None => "".to_string(),
                };
                let last_name = match schueler.last_name {
                    Some(l) => l,
                    None => "".to_string(),
                };
                return Ok(schema::SimpleSchueler {
                    id,
                    first_name,
                    last_name,
                });
            }
            Err(_e) => return Err(500),
        }
    }

    pub async fn get_dosb_task_for_schueler(
        id: i32,
        db: &SqlitePool,
    ) -> Result<Vec<schema::PflichtKategorie>, i32> {
        if !check_schueler_id(&id) {
            return Err(400);
        }
        let query_response =
            sqlx::query_as!(model::PflichtKategorie, r#"SELECT DISTINCT kategorien.id as id, (versuch.kategorieId IS NOT NULL) AS done, kategorien.kateGroupId as group_id FROM kategorien
    INNER JOIN dosbKat b ON b.katId = kategorien.id
    INNER JOIN katGroups ON kategorien.kateGroupId = katGroups.id
    INNER JOIN schueler ON schueler.age = b.age AND schueler.gesch = b.gesch
    LEFT JOIN versuch ON schueler.id = versuch.schuelerId AND kategorien.id = versuch.kategorieId AND versuch.isReal
    WHERE schueler.id = ? ORDER BY kategorien.kateGroupId;"#, id)
                .fetch_all(db)
                .await;

        return if query_response.iter().len() == 0 {
            Err(404)
        } else {
            match query_response {
                Ok(kategorien) => {
                    let note_responses = kategorien
                        .into_iter()
                        .map(|kategorie| pflicht_kat_model2schema(kategorie))
                        .collect::<Vec<schema::PflichtKategorie>>();
                    Ok(note_responses)
                }
                Err(e) => {
                    print!("{}", e);
                    Err(500)
                }
            }
        };
    }

    pub async fn get_bjs_task_for_schueler(
        id: i32,
        db: &SqlitePool,
    ) -> Result<Vec<schema::PflichtKategorie>, i32> {
        if !check_schueler_id(&id) {
            return Err(400);
        }
        let query_response =
            sqlx::query_as!(model::PflichtKategorie, r#"SELECT DISTINCT kategorien.id as id, (versuch.kategorieId IS NOT NULL) AS done, kategorien.kateGroupId as group_id FROM kategorien
    INNER JOIN bjsKat b ON b.katId = kategorien.id
    INNER JOIN katGroups ON kategorien.kateGroupId = katGroups.id
    INNER JOIN schueler ON schueler.age = b.age AND schueler.gesch = b.gesch
    LEFT JOIN versuch ON schueler.id = versuch.schuelerId AND kategorien.id = versuch.kategorieId AND versuch.isReal
    WHERE schueler.id = ? ORDER BY kategorien.kateGroupId;"#, id)
                .fetch_all(db)
                .await;

        return if query_response.iter().len() == 0 {
            Err(404)
        } else {
            match query_response {
                Ok(kategorien) => {
                    let note_responses = kategorien
                        .into_iter()
                        .map(|kategorie| pflicht_kat_model2schema(kategorie))
                        .collect::<Vec<schema::PflichtKategorie>>();
                    Ok(note_responses)
                }
                Err(e) => {
                    print!("{}", e);
                    Err(500)
                }
            }
        };
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
                if !(5..20).contains(&age) {
                    result.age_invalid.push(schueler);
                    continue;
                }
            } else if schueler.bday.is_some() && schueler.bday.clone().unwrap() != "-1" {
                println!("bday");
                let b_day_str = schueler.bday.clone().unwrap();
                let now = (SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    / 31536000) as u64
                    + 1970;
                println!("start regex");
                let re = Regex::new("/(20|19)[0-9][0-9]/gm").unwrap();
                let b_day = match re.find(b_day_str.as_str()) {
                    Some(b) => b.as_str().parse::<u64>().unwrap(),
                    None => {
                        result.age_invalid.push(schueler);
                        continue;
                    }
                };
                age = (now - b_day) as i8;
                println!("Age: {} - Now: {} - b_day: {}", age, now, b_day);
                if !(5..20).contains(&age) {
                    result.age_invalid.push(schueler);
                    continue;
                }
            } else {
                println("no age given");
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
    ) -> Result<Vec<schema::NormVersuch>, i32> {
        let versuch_result = sqlx::query_as!(model::NormVersuch, r#"
    SELECT id, schuelerId as schueler_id, kategorieId as kategorie_id, wert, mTime as ts_recording, isReal as is_real FROM versuch WHERE schuelerId = ? AND kategorieId = ?
    "#, id, kat_id).fetch_all(db).await;

        let versuche = match versuch_result {
            Ok(v) => v,
            Err(_e) => return Err(500),
        };

        Ok(
            futures::future::join_all(versuche.into_iter().map(|v| calc_norm_versuch(v, &db)))
                .await,
        )
    }

    pub async fn get_top_versuch_by_kat(
        id: i32,
        kat_id: i32,
        db: &SqlitePool,
    ) -> Result<schema::NormVersuch, i32> {
        let all_versuche_result = sqlx::query_as!(model::NormVersuch, r#"
        SELECT * FROM (
        SELECT versuch.id as id, schuelerId as schueler_id, kategorieId as kategorie_id, MIN(wert) as wert, mTime as ts_recording, isReal as is_real FROM versuch -- For Sprint and Ausdauer
            INNER JOIN kategorien ON kategorieId = kategorien.id
            WHERE kategorien.kateGroupId IN (1, 4) AND isReal = true GROUP BY versuch.schuelerId, kategorien.kateGroupId
        UNION 
        SELECT versuch.id as id, schuelerId as schueler_id, kategorieId as kategorie_id, MAX(wert) as wert, mTime as ts_recording, isReal as is_real FROM versuch -- For Sprung and Wurf/Stoß
            INNER JOIN kategorien ON kategorieId = kategorien.id
            WHERE kategorien.kateGroupId IN (2, 3) AND isReal = true GROUP BY versuch.schuelerId, kategorien.kateGroupId
        ) WHERE schueler_id = ? AND kategorie_id = ?
        "#, id, kat_id).fetch_one(db).await;

        let all_versuche_model = match all_versuche_result {
            Ok(r) => r,
            Err(_e) => return Err(500),
        };

        Ok(calc_norm_versuch(all_versuche_model, db).await)
    }

    pub async fn get_top_versuch_in_bjs(
        id: i32,
        db: &SqlitePool,
    ) -> Result<Vec<schema::NormVersuch>, i32> {
        let kat_list_result = sqlx::query_as!(
            model::KatId,
            r#"
        SELECT katId as id FROM bjsKat
        INNER JOIN schueler ON schueler.age = bjsKat.age AND schueler.gesch = bjsKat.gesch
        WHERE schueler.id = ?
        "#,
            id
        )
        .fetch_all(db)
        .await;

        let kat_list = match kat_list_result {
            Ok(r) => r,
            Err(_e) => return Err(500),
        };

        // get max and min for each
        let mut top_list: Vec<schema::NormVersuch> = Vec::new();
        for kat in kat_list {
            let top = get_top_versuch_by_kat(id, kat.id.unwrap() as i32, db).await;
            if top.is_ok() {
                top_list.push(top.unwrap());
            }
        }
        Ok(top_list)
    }

    pub async fn get_top_versuch_in_dosb(
        id: i32,
        db: &SqlitePool,
    ) -> Result<Vec<schema::NormVersuch>, i32> {
        let kat_list_result = sqlx::query_as!(
            model::KatId,
            r#"
        SELECT katId as id FROM dosbKat
        INNER JOIN schueler ON schueler.age = dosbKat.age AND schueler.gesch = dosbKat.gesch
        WHERE schueler.id = ?
        "#,
            id
        )
        .fetch_all(db)
        .await;

        let kat_list = match kat_list_result {
            Ok(r) => r,
            Err(_e) => return Err(500),
        };

        // get max and min for each
        let mut top_list: Vec<schema::NormVersuch> = Vec::new();
        for kat in kat_list {
            let top = get_top_versuch_by_kat(id, kat.id.unwrap() as i32, db).await;
            if top.is_ok() {
                top_list.push(top.unwrap());
            }
        }
        Ok(top_list)
    }

    pub async fn get_bjs_kat_groups(id: i32, db: &SqlitePool) -> Vec<Vec<i32>> {
        let mut result: Vec<Vec<i32>> = Vec::new();

        let query_result = sqlx::query_as!(
            model::KatGroup,
            r#"
        SELECT katId as id, kateGroupId as group_id FROM bjsKat
        INNER JOIN schueler ON schueler.age = bjsKat.age AND schueler.gesch = bjsKat.gesch
        INNER JOIN kategorien ON kategorien.id = katId
        WHERE schueler.id = ? ORDER BY kateGroupId
        "#,
            id
        )
        .fetch_all(db)
        .await
        .unwrap();

        let mut last_group_id: i32 = -1;
        let mut current_group: Vec<i32> = Vec::new();

        for k in query_result {
            if k.group_id.unwrap() as i32 != last_group_id {
                if !current_group.is_empty() {
                    result.push(current_group);
                    current_group = Vec::new();
                }
                last_group_id = k.group_id.unwrap() as i32;
            }
            current_group.push(k.id.unwrap() as i32);
        }
        result.push(current_group);

        return result;
    }

    pub async fn get_bjs_points(id: i32, db: &SqlitePool) -> Result<i32, i32> {
        let kat_groups = get_bjs_kat_groups(id, db).await;
        let versuche = get_top_versuch_in_bjs(id, db).await?;

        if versuche.is_empty() {
            return Ok(0);
        }

        let mut result_vec: Vec<i32> = Vec::new();
        let mut tmp_points: i32 = 0;

        for g in kat_groups {
            for v in versuche.to_owned() {
                if g.contains(&(v.kategorie_id as i32)) && v.punkte > tmp_points as i64 {
                    tmp_points = v.punkte as i32;
                }
            }
            result_vec.push(tmp_points);
            tmp_points = 0;
        }
        result_vec.sort();
        let mut lowest = 0;
        if result_vec.len() >= 4 {
            lowest = result_vec.first().unwrap().to_owned();
        }
        Ok((result_vec.into_iter().sum::<i32>()) - lowest)
    }

    pub async fn needs_kat(
        schueler_id: i32,
        kategorie_id: i32,
        db: &SqlitePool,
    ) -> Result<schema::NeedsKat, i32> {
        let dosb_result = sqlx::query_as!(model::NeedsKat, r#"
        SELECT (dosbKat.gesch NOT NULL) as need FROM schueler
        LEFT JOIN dosbKat ON schueler.gesch = dosbKat.gesch AND dosbKat.age = schueler.age AND dosbKat.katId = ?
        WHERE schueler.id = ?
        "#, kategorie_id, schueler_id).fetch_one(db).await;
        let dosb = match dosb_result {
            Ok(r) => r,
            Err(_e) => return Err(404),
        };

        let bjs_result = sqlx::query_as!(model::NeedsKat, r#"
        SELECT (bjsKat.gesch NOT NULL) as need FROM schueler
        LEFT JOIN bjsKat ON schueler.gesch = bjsKat.gesch AND bjsKat.age = schueler.age AND bjsKat.katId = ?
        WHERE schueler.id = ?
        "#, kategorie_id, schueler_id).fetch_one(db).await;
        let bjs = match bjs_result {
            Ok(r) => r,
            Err(_e) => return Err(404),
        };

        Ok(schema::NeedsKat {
            dosb: dosb.need != 0,
            bjs: bjs.need != 0,
        })
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

    pub async fn get_versuch_by_id(id: i32, db: &SqlitePool) -> Result<schema::NormVersuch, i32> {
        let versuch_result = sqlx::query_as!(model::NormVersuch, r#"
    SELECT id, schuelerId as schueler_id, kategorieId as kategorie_id, wert, mTime as ts_recording, isReal as is_real FROM versuch WHERE id = ?
    "#, id).fetch_one(db).await;
        match versuch_result {
            Ok(r) => Ok(calc_norm_versuch(r, &db).await),
            Err(_e) => Err(404),
        }
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
        SELECT id, name, lauf, einheit, maxVers as max_vers, digits_before, digits_after, kateGroupId as kat_group_id FROM kategorien WHERE id = ?
        "#, id).fetch_one(db).await;
        return kategorie_model2schema(result.unwrap());
    }

    pub async fn calc_points(versuch: schema::SimpleVersuch, db: &SqlitePool) -> i32 {
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

    fn pflicht_kat_model2schema(m: model::PflichtKategorie) -> schema::PflichtKategorie {
        schema::PflichtKategorie {
            id: m.id.unwrap(),
            done: m.done != 0,
            group_id: m.group_id.unwrap(),
        }
    }

    async fn calc_norm_versuch(v: model::NormVersuch, db: &SqlitePool) -> schema::NormVersuch {
        schema::NormVersuch {
            id: v.id.unwrap(),
            schueler_id: v.schueler_id.unwrap(),
            kategorie_id: v.kategorie_id.unwrap(),
            wert: (v.wert.unwrap() * 100.0).round() / 100.0,
            punkte: calc_points(
                schema::SimpleVersuch {
                    schueler_id: v.schueler_id.unwrap() as i32,
                    wert: v.wert.unwrap() as f32,
                    kategorie_id: v.kategorie_id.unwrap() as i32,
                },
                db,
            )
            .await
            .into(),
            ts_recording: v.ts_recording.unwrap(),
            is_real: v.is_real.unwrap(),
        }
    }

    fn kategorie_model2schema(m: model::Kategorie) -> schema::Kategorie {
        schema::Kategorie {
            id: m.id.unwrap(),
            name: m.name.unwrap(),
            lauf: m.lauf.unwrap(),
            einheit: m
                .einheit
                .unwrap()
                .chars()
                .next()
                .expect("no Unit was given"),
            max_vers: m.max_vers.unwrap(),
            digits_before: m.digits_before.unwrap(),
            digits_after: m.digits_after.unwrap(),
            kat_group_id: m.kat_group_id.unwrap(),
        }
    }

    fn check_schueler_id(id: &i32) -> bool {
        return id < &9999 && id > &1000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time_test::time_test;

    async fn migrate_example_db() -> SqlitePool {
        let con = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!().run(&con).await.unwrap();
        return con;
    }

    #[actix_rt::test]
    async fn test_db_migration() {
        let _ = migrate_example_db().await;
    }

    #[actix_rt::test]
    async fn get_schueler() {
        let db = migrate_example_db().await;
        let test_schueler = interact::get_schueler(&1234, &db).await.unwrap();
        let result_schueler = schema::SimpleSchueler {
            id: 1234,
            first_name: String::from("Franz2"),
            last_name: String::from("Peterson"),
        };

        assert_eq!(result_schueler, test_schueler);
    }

    #[actix_rt::test]
    async fn get_dosb_schueler() {
        let db = migrate_example_db().await;
        let test_result = interact::get_dosb_task_for_schueler(1234, &db)
            .await
            .unwrap();
        assert_eq!(
            r#"[{"done":false,"group_id":1,"id":3},{"done":false,"group_id":2,"id":6},{"done":false,"group_id":2,"id":7},{"done":false,"group_id":3,"id":10},{"done":false,"group_id":4,"id":4}]"#,
            format!("{}", serde_json::json!(test_result))
        );
    }

    #[actix_rt::test]
    async fn calc_points() {
        let db = migrate_example_db().await;

        // 50m Lauf 8.4 -> 324 (w)
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 1234,
                    wert: 8.4,
                    kategorie_id: 1
                },
                &db
            )
            .await,
            324
        );

        // 800m Lauf 180 -> 329 (m)
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 3809,
                    wert: 180.0,
                    kategorie_id: 4
                },
                &db
            )
            .await,
            329
        );

        // 200g Ballwurf 27.5 -> 266 (m)
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 3809,
                    wert: 27.5,
                    kategorie_id: 10
                },
                &db
            )
            .await,
            266
        );

        // kat out of range
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 3809,
                    wert: 27.5,
                    kategorie_id: 123
                },
                &db
            )
            .await,
            -404
        );

        // schuelerId out of range
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 309,
                    wert: 27.5,
                    kategorie_id: 3
                },
                &db
            )
            .await,
            -404
        );
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 35409,
                    wert: 27.5,
                    kategorie_id: 3
                },
                &db
            )
            .await,
            -404
        );

        // points lower than 0 or higher than 900
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 3809,
                    wert: 3.0,
                    kategorie_id: 3
                },
                &db
            )
            .await,
            -406
        );
        assert_eq!(
            interact::calc_points(
                schema::SimpleVersuch {
                    schueler_id: 3809,
                    wert: 3.0,
                    kategorie_id: 10
                },
                &db
            )
            .await,
            0
        );
    }

    #[actix_rt::test]
    async fn add_get_versuch() {
        let db = migrate_example_db().await;
        let v1 = schema::SimpleVersuch {
            schueler_id: 1234,
            wert: 15.0,
            kategorie_id: 10,
        };
        let v1_copy = schema::SimpleVersuch {
            schueler_id: 1234,
            wert: 15.0,
            kategorie_id: 10,
        };
        let v1_id = interact::add_versuch(v1, "ABCA".to_string(), &db)
            .await
            .unwrap();
        let get_v1 = interact::get_versuch_by_id(v1_id, &db).await.unwrap();

        assert_eq!(&v1_copy.schueler_id, &(get_v1.schueler_id as i32));
        assert_eq!(&v1_copy.wert, &(get_v1.wert as f32));
        assert_eq!(&v1_copy.kategorie_id, &(get_v1.kategorie_id as i32));
    }

    #[actix_rt::test]
    async fn top_versuch_by_kat() {
        let db = migrate_example_db().await;
        let v1 = schema::SimpleVersuch {
            schueler_id: 1234,
            wert: 15.0,
            kategorie_id: 10,
        };
        let current_top = schema::SimpleVersuch {
            schueler_id: 1234,
            wert: 15.0,
            kategorie_id: 10,
        };
        let _ = interact::add_versuch(v1, "ABCA".to_string(), &db).await;

        // only one try
        let test_top = interact::get_top_versuch_by_kat(1234, 10, &db)
            .await
            .unwrap();
        assert_eq!(&current_top.schueler_id, &(test_top.schueler_id as i32));
        assert_eq!(&current_top.wert, &(test_top.wert as f32));
        assert_eq!(&current_top.kategorie_id, &(test_top.kategorie_id as i32));

        let current_top = schema::SimpleVersuch {
            schueler_id: 1234,
            wert: 17.0,
            kategorie_id: 10,
        };
        let _ = interact::add_versuch(
            schema::SimpleVersuch {
                schueler_id: 1234,
                wert: 17.0,
                kategorie_id: 10,
            },
            "AWDF".to_string(),
            &db,
        )
        .await;

        // new Top try
        let test_top = interact::get_top_versuch_by_kat(1234, 10, &db)
            .await
            .unwrap();
        assert_eq!(&current_top.schueler_id, &(test_top.schueler_id as i32));
        assert_eq!(&current_top.wert, &(test_top.wert as f32));
        assert_eq!(&current_top.kategorie_id, &(test_top.kategorie_id as i32));

        let _ = interact::add_versuch(
            schema::SimpleVersuch {
                schueler_id: 1234,
                wert: 14.0,
                kategorie_id: 10,
            },
            "AWDF".to_string(),
            &db,
        )
        .await;

        // new Top try
        time_test!();
        let test_top = interact::get_top_versuch_by_kat(1234, 10, &db)
            .await
            .unwrap();
        assert_eq!(&current_top.schueler_id, &(test_top.schueler_id as i32));
        assert_eq!(&current_top.wert, &(test_top.wert as f32));
        assert_eq!(&current_top.kategorie_id, &(test_top.kategorie_id as i32));
    }

    #[actix_rt::test]
    async fn top_versuche_and_points_by_bjs() {
        let db = migrate_example_db().await;
        let mut versuch_ids: Vec<i32> = Vec::new();
        let _ = interact::add_versuch(
            schema::SimpleVersuch {
                schueler_id: 1234,
                wert: 10.7,
                kategorie_id: 1,
            },
            "AWDF".to_string(),
            &db,
        )
        .await;
        versuch_ids.push(
            interact::add_versuch(
                schema::SimpleVersuch {
                    schueler_id: 1234,
                    wert: 14.0,
                    kategorie_id: 3,
                },
                "AWDF".to_string(),
                &db,
            )
            .await
            .unwrap(),
        );
        let _ = interact::add_versuch(
            schema::SimpleVersuch {
                schueler_id: 1234,
                wert: 15.0,
                kategorie_id: 3,
            },
            "AWDF".to_string(),
            &db,
        )
        .await;
        versuch_ids.push(
            interact::add_versuch(
                schema::SimpleVersuch {
                    schueler_id: 1234,
                    wert: 3.7,
                    kategorie_id: 7,
                },
                "AWDF".to_string(),
                &db,
            )
            .await
            .unwrap(),
        );
        versuch_ids.push(
            interact::add_versuch(
                schema::SimpleVersuch {
                    schueler_id: 1234,
                    wert: 1.4,
                    kategorie_id: 6,
                },
                "AWDF".to_string(),
                &db,
            )
            .await
            .unwrap(),
        );
        versuch_ids.push(
            interact::add_versuch(
                schema::SimpleVersuch {
                    schueler_id: 1234,
                    wert: 14.0,
                    kategorie_id: 10,
                },
                "AWDF".to_string(),
                &db,
            )
            .await
            .unwrap(),
        );
        versuch_ids.push(
            interact::add_versuch(
                schema::SimpleVersuch {
                    schueler_id: 1234,
                    wert: 702.0,
                    kategorie_id: 5,
                },
                "AWDF".to_string(),
                &db,
            )
            .await
            .unwrap(),
        );
        let _ = interact::add_versuch(
            schema::SimpleVersuch {
                schueler_id: 3809,
                wert: 242.0,
                kategorie_id: 4,
            },
            "AWDF".to_string(),
            &db,
        )
        .await;

        //time_test!();
        //let mut result_ids: Vec<i32> = Vec::new();

        let result = interact::get_top_versuch_in_bjs(1234, &db).await.unwrap();
        let mut result_ids: Vec<i32> = result.into_iter().map(|v| v.id as i32).collect();

        result_ids.sort();
        versuch_ids.sort();
        assert_eq!(versuch_ids, result_ids);

        assert_eq!(1126, interact::get_bjs_points(1234, &db).await.unwrap());
    }

    #[actix_rt::test]
    async fn get_bjs_kat_groups() {
        let db = migrate_example_db().await;
        let test = interact::get_bjs_kat_groups(1234, &db).await;
        assert_eq!(test, vec![vec![2, 3], vec![6, 7], vec![10], vec![4, 5]]);
    }

    #[actix_rt::test]
    async fn need_kat() {
        let db = migrate_example_db().await;
        let kat3 = interact::needs_kat(1234, 3, &db).await.unwrap();
        assert!(kat3.bjs);
        assert!(kat3.dosb);

        let kat5 = interact::needs_kat(1234, 5, &db).await.unwrap();
        assert!(kat5.bjs);
        assert!(!kat5.dosb);

        let kat1 = interact::needs_kat(1234, 1, &db).await.unwrap();
        assert!(!kat1.bjs);
        assert!(!kat1.dosb);
    }
}
