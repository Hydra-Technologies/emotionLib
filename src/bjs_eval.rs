//! This module is used to evaluate the results of the student with the specification of the BJS
use sqlx::sqlite::SqlitePool;
use actix_web::HttpResponse;
use serde::Serialize;
use crate::model::{Attempt, Category};
use std::collections::HashMap;
use log::debug;


#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BJSAbzeichen {
    None,
    Teilnehmer,
    Sieger,
    Ehren
}

pub struct BJSEvaluator<'a> {
    pub db: &'a SqlitePool
}
impl BJSEvaluator<'_> {
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

        let mut top_attempts: HashMap<i64,(i64, Attempt)> = HashMap::new();
        for a in used_attempts {
            let points = self.calculate_points(gender, &a).await?;

            if let Some((last_points, _)) = top_attempts.get(&a.category) {
                if last_points < &points {
                    top_attempts.insert(a.category, (points, a));
                }
                
            } else {
                top_attempts.insert(a.category, (points, a));
            }
        }


        // add the best attempts to the list
        return Ok(top_attempts.into_iter().map(|a| a.1.1).collect())
    }

    pub async fn calculate_points(&self,gender: char, att: &Attempt) -> Result<i64, HttpResponse> {
        let gender_string = gender.to_string();
        // get the a, c numbers, as well as the running and if exists the distance of the category
        let vars = match sqlx::query!(r#"
            SELECT a, c, running, distance FROM category
            INNER JOIN form_vars ON category_id = category.id
            WHERE category.id = ? AND gender = ?
        "#, att.category, gender_string).fetch_one(self.db).await {
            Ok(r) => r,
            Err(e) => return Err(InternalServerf!("Error while fetching the vars to calculate the points ({})", e))
        };
        self::BJSEvaluator::calculate_points_with_know_vars(vars.a, vars.c, vars.running, vars.distance, att)
    }
    
    /// for efficiency reason we have an extra function for this.
    /// That way we dont have to fetch the a and c numbers more often than actually needed
   fn calculate_points_with_know_vars(a: f64, c: f64, running: bool, distance: Option<i64>, att: &Attempt) -> Result<i64, HttpResponse> {
        let points: i64 = if running {
            let distance = match distance {
                Some(r) => r,
                None => return Err(InternalServerf!("Distance is zero while running is true for category {}", att.category))
            };

            let supplement = 
                if distance <= 300 {
                    0.24
                } else if distance <= 400 {
                    0.14
                } else {
                    0.0
                };
            
            ((distance as f64/(att.result+supplement) - a)/c) as i64
        } else {
            ((att.result.sqrt() - a)/c) as i64
        };

        if points < 0 {
            return Ok(0)
        }

        return Ok(points)
    }

   pub async fn calculate_points_sum(&self,age: i64, gender: char, attempts: Vec<Attempt>) -> Result<i64,HttpResponse> {
        let gender_string = gender.to_string();
        let top_attempts = self.get_top_attempts(age,gender,attempts).await?;

        let mut top_points = vec![0,0,0,0];
        for att in top_attempts {
            // get the group
            let cat_info= match sqlx::query!(r#"
                    SELECT a, c, running, distance, category_group_id FROM category
                    INNER JOIN form_vars ON category_id = category.id
                    WHERE category.id = ? AND gender = ?
                "#, att.category, gender_string).fetch_one(self.db).await {
                    Ok(r) => r,
                    Err(sqlx::Error::RowNotFound) => return Err(NotFoundf!("The Category with id {} was not found in bjs Database", att.category)),
                    Err(e) => return Err(InternalServerf!("Something went wrong while fetching the categoryid {} in the bjs Database Error: {}", att.category, e))
                };

            // Wooho worth it
            let points = self::BJSEvaluator::calculate_points_with_know_vars(cat_info.a, cat_info.c, cat_info.running, cat_info.distance, &att)?;

            if top_points[(cat_info.category_group_id -1) as usize ] < points {
                top_points[(cat_info.category_group_id -1) as usize ] = points;
            }
        }

        // pick the three best
        let mut point_sum = 0;
        let mut min = i64::MAX;
        for p in top_points {
            point_sum += p;
            if p < min {
                min = p
            }
        }
        point_sum -= min;
        return Ok(point_sum);
   }

    pub async fn get_medal(&self, age: i64, gender: char, attempts: Vec<Attempt>) -> Result<BJSAbzeichen,HttpResponse> {
        if attempts.len() == 0 {
            return Ok(BJSAbzeichen::None)
        }

        let gender_string = gender.to_string();
        // chech if at least 3 categories are done
        let done_categories = attempts.iter().map(|a| a.category).collect();
        let missing_by_group= self.get_missing_categorys(age, gender, done_categories).await?;
        let mut num_of_done_categories = 0;
        for group in missing_by_group {
            if group.len() == 0 {
                num_of_done_categories+=1;
            }
        }
        // if you dont have enough categorygroups you dont get a medal
        if num_of_done_categories < 3 {
            return Ok(BJSAbzeichen::Teilnehmer)
        }

        let point_sum = self.calculate_points_sum(age, gender, attempts).await?;

        // check what medal that is
        let thresholds = match sqlx::query!("SELECT winner, honor FROM points_eval WHERE age = ? AND gender = ?", age, gender_string).fetch_one(self.db).await {
            Ok(r) => r,
            Err(sqlx::Error::RowNotFound) => return Err(NotFoundf!("The age group age: {} and gender {} was not found in the bjs points_eval table", age, gender)),
            Err(e) => return Err(InternalServerf!("There was an error while fetching the age group from bjs Error: {}", e))
        };

        return if point_sum < thresholds.winner {
            Ok(BJSAbzeichen::Teilnehmer)
        } else if point_sum < thresholds.honor {
            Ok(BJSAbzeichen::Sieger)
        } else {
            Ok(BJSAbzeichen::Ehren)
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
        let eval = BJSEvaluator {
            db: &SqlitePool::connect("testData/2025bjs.db").await.unwrap()
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
        assert_eq!(eval.calculate_points(gender, &attempts[0]).await.unwrap(), 304, "Weitsprung");
        assert_eq!(eval.calculate_points(gender, &attempts[1]).await.unwrap(), 216, "800m Lauf");
        assert_eq!(eval.calculate_points(gender, &attempts[2]).await.unwrap(), 363, "50m Lauf");
        assert_eq!(eval.calculate_points(gender, &attempts[3]).await.unwrap(), 200, "80g Schlagball");

        // now for the medal of all
        assert_eq!(eval.get_medal(age, gender, attempts).await.unwrap(), BJSAbzeichen::Ehren);
    }

    #[sqlx::test]
    async fn schueler_5243_2025() {
        // create evaluator
        let eval = BJSEvaluator {
            db: &SqlitePool::connect("testData/2025bjs.db").await.unwrap()
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
        assert_eq!(eval.calculate_points(gender, &attempts[0]).await.unwrap(), 306, "Weitsprung");
        // there is no Standweitsprung in bjs
        assert!(eval.calculate_points(gender, &attempts[1]).await.is_err(), "Standweitsprung");
        assert_eq!(eval.calculate_points(gender, &attempts[2]).await.unwrap(), 278, "800m Lauf");
        assert_eq!(eval.calculate_points(gender, &attempts[3]).await.unwrap(), 366, "50m Lauf");
        assert_eq!(eval.calculate_points(gender, &attempts[4]).await.unwrap(), 247, "Hochsprung");
        assert_eq!(eval.calculate_points(gender, &attempts[5]).await.unwrap(), 340, "80g Schlagball");

        // now for the medal of all
        assert_eq!(eval.get_medal(age, gender, attempts).await.unwrap(), BJSAbzeichen::Sieger);
    }

    #[sqlx::test]
    async fn category_group_missing() {
        // create evaluator
        let eval = BJSEvaluator{
            db: &SqlitePool::connect("testData/2025bjs.db").await.unwrap()
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
                result: 2.0
            }
        );

        // there are no categorys missing
        let missing = eval.get_missing_categorys(age, gender, 
            attempts.iter().map(|a| a.category).collect()).await.unwrap();
        println!("Missing: {:?}", missing);
        assert_eq!(missing[0].len(), 0);
        assert!(missing[1].len() > 0);
        assert!(missing[2].len() > 0);
        assert_eq!(missing[3].len(), 0);

        // now for the medal of all
        assert_eq!(eval.get_medal(age, gender, attempts).await.unwrap(), BJSAbzeichen::Teilnehmer);
    }
}
