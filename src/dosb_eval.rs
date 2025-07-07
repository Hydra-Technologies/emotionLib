//! This module is used to evaluate the results of the student with the specification of the DOSB
use sqlx::sqlite::SqlitePool;
use actix_web::HttpResponse;
use serde::Serialize;
use crate::model::{Attempt, Category};
use std::collections::HashMap;
use log::debug;


#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DOSBAbzeichen {
    None = 0,
    Bronze = 1,
    Silver = 2,
    Gold = 3,
}

pub struct DOSBEvaluator<'a> {
    pub db: &'a SqlitePool
}
impl DOSBEvaluator<'_> {

    pub async fn get_needed_categorys(&self, age: i64, gender: char) -> Result<Vec<Category>, HttpResponse> {
        let gender_string = gender.to_string();
        return match sqlx::query_as!(Category, r#"
            SELECT category_id as id, category_group_id as group_id FROM mand_category
            INNER JOIN category ON category_id = category.id
            WHERE age = ? AND gender = ?"#, 
            age, gender_string)
            .fetch_all(self.db).await {
                Ok(r) => Ok(r),
                Err(e) => return Err(InternalServerf!("Error while fetching the categories ({})", e))
        };
    }
    /// get the missing categories while some are still finished
    pub async fn get_missing_categorys(&self, age: i64, gender: char, done_categories: Vec<i64>) -> Result<Vec<Vec<i64>>, HttpResponse> {
        let need_categories = self.get_needed_categorys(age, gender).await?;
        // filter the categories for those that where actually needed
        let mut done_needed_categories= vec![];
        for k in &need_categories {
            if done_categories.contains(&k.id) {
                done_needed_categories.push(k);
            }
        }

        // find out wich groups are done
        let mut done_groups = vec![false, false, false, false];
        for k in &done_needed_categories {
            done_groups[(k.group_id - 1) as usize]  = true;
        }

        // order the groups into the right vetors if the group is not done
        let mut missing = vec![vec![],vec![],vec![],vec![]];
        for k in need_categories {
            if !done_groups[(k.group_id - 1) as usize] {
                missing[(k.group_id - 1) as usize].push(k.id);
            }
        }

        return Ok(missing);
    }

    pub async fn get_top_attempts(&self, age: i64, gender: char, attempts: Vec<Attempt>) -> Result<Vec<Attempt>,HttpResponse> {

        // filter for the attempts that are actually needed and calculate their medals
        let needed_categories = self.get_needed_categorys(age, gender).await?;
        let needed_categories_ids: Vec<i64> = needed_categories.iter().map(|c| c.id).collect();
        let used_attempts: Vec<Attempt> = attempts.into_iter()
            .filter(|a| needed_categories_ids.contains(&a.category))
            .collect();

        let mut top_attempts: HashMap<i64,Attempt> = HashMap::new();
        for a in used_attempts {
            // get the group id and if bigger is better
            let q = match sqlx::query!(r#"
                SELECT bronze, gold FROM mand_category
                WHERE category_id = ?
                LIMIT 1
            "#, a.category)
                .fetch_one(self.db).await {
                    Ok(r) => r,
                    Err(sqlx::Error::RowNotFound) => return Err(NotFoundf!("The Category {}, was not found in the dosb database", a.category)),
                    Err(e) => return Err(InternalServerf!("Something went wrong while fetching the category {} Error: {}", a.category, e))
            };

            if let Some(last_attempt) = top_attempts.get(&a.category) {
                if q.gold > q.bronze {
                    // bigger is better
                    if last_attempt.result < a.result {
                        top_attempts.insert(a.category, a);
                    }
                } else {
                    if last_attempt.result > a.result {
                        top_attempts.insert(a.category, a);
                    }
                }
                
            } else {
                top_attempts.insert(a.category, a);
            }
        }


        // add the best attempts to the list
        return Ok(top_attempts.into_iter().map(|a| a.1).collect())
    }

    pub async fn get_medal_for_attempt(&self, age: i64, gender: char, att: &Attempt) -> Result<DOSBAbzeichen, HttpResponse> {
        let gender_string = gender.to_string();
        // get thresholds
        let threshholds = match sqlx::query!(r#"
            SELECT gold, silver, bronze FROM mand_category WHERE age = ? AND gender = ? AND category_id = ?
        "#, age, gender_string, att.category)
            .fetch_one(self.db).await {
                Ok(e) => e,
                Err(sqlx::Error::RowNotFound) => return Err(NotFoundf!("age, gender or the category where not found in the dosb database (Cat: {},age:{},gesch:{})", att.category,age,gender)),
                Err(e) => return Err(InternalServerf!("Something went wrong while fetching the thresholds for the category {} with age {} and gender {} \n Error: {}", att.category, age, gender, e))
            };

        // check if bigger is better or the other way around
        if threshholds.bronze < threshholds.silver {
            // if so we can check one by one wich medal is applied
            if att.result  < threshholds.bronze - 0.01 {
                return Ok(DOSBAbzeichen::None)
            }

            if att.result < threshholds.silver - 0.01 {
                return Ok(DOSBAbzeichen::Bronze)
            }

            if att.result < threshholds.gold - 0.01 {
                return Ok(DOSBAbzeichen::Silver)
            }

            return Ok(DOSBAbzeichen::Gold)
        } else {
            // if so we can check one by one wich medal is applied
            if att.result > threshholds.bronze + 0.01 {
                return Ok(DOSBAbzeichen::None)
            }

            if att.result > threshholds.silver + 0.01 {
                return Ok(DOSBAbzeichen::Bronze)
            }

            if att.result > threshholds.gold + 0.01 {
                return Ok(DOSBAbzeichen::Silver)
            }

            return Ok(DOSBAbzeichen::Gold)
        }
    }
    pub async fn calculate_points(&self, age: i64, gender: char, attempts: Vec<Attempt>) -> Result<u8,HttpResponse> {
        let top_attempts = self.get_top_attempts(age, gender, attempts).await?;

        // get category group
        let mut attemps_with_group: HashMap<i64, (Attempt, DOSBAbzeichen)> = HashMap::new();
        for att in top_attempts  {
            let id = match sqlx::query!(r#"
                    SELECT category_group_id FROM category WHERE id = ?
                "#, att.category).fetch_one(self.db).await {
                    Ok(r) => r.category_group_id,
                    Err(sqlx::Error::RowNotFound) => return Err(NotFoundf!("The Category with id {} was not found in dosb cat db", att.category)),
                    Err(e) => return Err(InternalServerf!("Something went wrong while fetching the category group id {} Error: {}", att.category, e))
                };

            let abzeichen = self.get_medal_for_attempt(age, gender, &att).await?;
            if let Some(last_attempt) = attemps_with_group.get(&id) {
                if (last_attempt.1 as u8) < (abzeichen as u8) {
                    attemps_with_group.insert(id,(att,abzeichen));
                }
            } else {
                attemps_with_group.insert(id,(att,abzeichen));
            }
        }

        // add the medals 
        let mut medal_sum = 0;
        for att in attemps_with_group.into_iter().map(|c| c.1.1) {
            medal_sum += att as u8;
        }

        return Ok(medal_sum);
    }

    pub async fn get_medal(&self, age: i64, gender: char, attempts: Vec<Attempt>) -> Result<DOSBAbzeichen,HttpResponse> {
        let done_categories = attempts.iter().map(|a| a.category).collect();
        let missing_by_group= self.get_missing_categorys(age, gender, done_categories).await?;
        if     missing_by_group[0].len() > 0 
            || missing_by_group[1].len() > 0 
            || missing_by_group[2].len() > 0 
            || missing_by_group[3].len() > 0 {
            debug!("There are still some categories missing"); 
            return Ok(DOSBAbzeichen::None);
        }

        let medal_sum = self.calculate_points(age, gender, attempts).await?;

        return if medal_sum < 4 {
            Ok(DOSBAbzeichen::None)
        } else if medal_sum < 8 {
            Ok(DOSBAbzeichen::Bronze)
        } else if medal_sum < 11 {
            Ok(DOSBAbzeichen::Silver)
        } else {
            Ok(DOSBAbzeichen::Gold)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn schueler_5716_2025() {
        // create evaluator
        let eval = DOSBEvaluator {
            db: &SqlitePool::connect("testData/2025dosb.db").await.unwrap()
        };

        let mut attempts = vec![];
        let age = 11;
        let gender = 'm';
        // Weitsprung
        attempts.push(
            Attempt {
                category: 4,
                result: 3.3
            }
        );
        // 800m Lauf
        attempts.push(
            Attempt {
                category: 14,
                result: 215.0
            }
        );
        // 50m Lauf
        attempts.push(
            Attempt {
                category: 1,
                result: 7.7
            }
        );
        // 80g Schlagball
        attempts.push(
            Attempt {
                category: 6,
                result: 25.0
            }
        );

        // there are no categorys missing
        let missing = eval.get_missing_categorys(age, gender, 
            attempts.iter().map(|a| a.category).collect()).await.unwrap();
        assert_eq!(missing[0].len(), 0);
        assert_eq!(missing[1].len(), 0);
        assert_eq!(missing[2].len(), 0);
        assert_eq!(missing[3].len(), 0);

        // now we check the medal for each of these
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[0]).await.unwrap(), DOSBAbzeichen::Gold, "Weitsprung");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[1]).await.unwrap(), DOSBAbzeichen::Gold, "800m Lauf");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[2]).await.unwrap(), DOSBAbzeichen::Gold, "50m Lauf");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[3]).await.unwrap(), DOSBAbzeichen::Silver, "80g Schlagball");

        // now for the medal of all
        assert_eq!(eval.get_medal(age, gender, attempts).await.unwrap(), DOSBAbzeichen::Gold);
    }

    #[sqlx::test]
    async fn schueler_5243_2025() {
        // create evaluator
        let eval = DOSBEvaluator {
            db: &SqlitePool::connect("testData/2025dosb.db").await.unwrap()
        };

        let mut attempts = vec![];
        let age = 13;
        let gender = 'w';
        // Weitsprung
        attempts.push(
            Attempt {
                category: 4,
                result: 3.0
            }
        );
        // Standweitsprung
        attempts.push(
            Attempt {
                category: 18,
                result: 1.8
            }
        );
        // 800m Lauf
        attempts.push(
            Attempt {
                category: 14,
                result: 209.0
            }
        );
        // 50m Lauf
        attempts.push(
            Attempt {
                category: 1,
                result: 8.0
            }
        );
        // Hochsprung
        attempts.push(
            Attempt {
                category: 5,
                result: 1.1
            }
        );
        // 80g Schlagball
        attempts.push(
            Attempt {
                category: 6,
                result: 25.0
            }
        );

        // there are no categorys missing
        let missing = eval.get_missing_categorys(age, gender, 
            attempts.iter().map(|a| a.category).collect()).await.unwrap();
        assert_eq!(missing[0].len(), 0);
        assert_eq!(missing[1].len(), 0);
        assert_eq!(missing[2].len(), 0);
        assert_eq!(missing[3].len(), 0);

        // now we check the medal for each of these
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[0]).await.unwrap(), DOSBAbzeichen::Bronze, "Weitsprung");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[1]).await.unwrap(), DOSBAbzeichen::Gold, "Standweitsprung");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[2]).await.unwrap(), DOSBAbzeichen::Gold, "800m Lauf");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[3]).await.unwrap(), DOSBAbzeichen::Gold, "50m Lauf");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[4]).await.unwrap(), DOSBAbzeichen::Gold, "Hochsprung");
        assert_eq!(eval.get_medal_for_attempt(age, gender, &attempts[5]).await.unwrap(), DOSBAbzeichen::Gold, "80g Schlagball");

        // now for the medal of all
        assert_eq!(eval.get_medal(age, gender, attempts).await.unwrap(), DOSBAbzeichen::Gold);
    }

    #[sqlx::test]
    async fn category_group_missing() {
        // create evaluator
        let eval = DOSBEvaluator {
            db: &SqlitePool::connect("testData/2025dosb.db").await.unwrap()
        };

        let mut attempts = vec![];
        let age = 13;
        let gender = 'w';
        // 800m Lauf
        attempts.push(
            Attempt {
                category: 14,
                result: 209.0
            }
        );
        // 50m Lauf
        attempts.push(
            Attempt {
                category: 1,
                result: 8.0
            }
        );
        // 80g Schlagball
        attempts.push(
            Attempt {
                category: 6,
                result: 25.0
            }
        );

        // there are no categorys missing
        let missing = eval.get_missing_categorys(age, gender, 
            attempts.iter().map(|a| a.category).collect()).await.unwrap();
        println!("Missing: {:?}", missing);
        assert_eq!(missing[0].len(), 0);
        assert_eq!(missing[1].len(), 0);
        assert!(missing[2].len() > 0);
        assert_eq!(missing[3].len(), 0);

        // now for the medal of all
        assert_eq!(eval.get_medal(age, gender, attempts).await.unwrap(), DOSBAbzeichen::None);
    }
}
