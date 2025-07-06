//! Here is all we need for authentication
//! 
//! The main idea is to set a macro for a endpoint like:
//! ```
//! use actix_web::{HttpRequest,HttpResponse};
//! //#[ensure_user] // uncomment this
//! pub async fn addEventCategory(req: HttpRequest, data: i64) -> HttpResponse{
//!     HttpResponse::Ok().into()
//! }
//! ```
//! Instead of the data int ther has to be a struct with a attribute named db with a SqlitePool
//!
//!
//! This should enforce that the user is a Admin, as well as a user varible, also it should expose
//! the event Varible

use crate::{Forbidden,NotFound,InternalServer, BadRequest, Unauthorized};
use sqlx::SqlitePool;
use actix_web::{HttpRequest, HttpResponse};
use sha256::digest;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::*;
use log::info;

/**
 * includes all the information to access the Database
 */
pub struct Event {
    pub id: String,
    pub name: String,
}

/**
 * The different kinds of request combinations that can be send.
 *
 * This user contains all the information that can be send in the Request
 */
pub enum RequestUser {
    Admin{api_key: String},
    AdminWithEvent{api_key: String, event_id: String},
    TmpUser{api_key: String},
}

/**
 * The differrent kinds of Users. With all the relevant information to check the authentication
 *
 * This user shall exist and have a valid session
 */
pub enum AuthUser {
    Admin{api_key: String},
    AdminWithEvent{api_key: String, event_id: String},
    TmpUser{id: String, api_key: String, event_id: String},
    NotApprovedTmpUser{id: String, api_key: String}
}

/**
 * Get the Userver from the Database. While doing this the validity of the session is checked
 */
pub async fn get_user(req: &HttpRequest, db: &SqlitePool) -> Result<AuthUser, HttpResponse> {
    let user = req2user(req)?;
    // get Current time
    // this is used to check if the Session is valid and update it to the new number
    let current_timestamp: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    match user {
        RequestUser::TmpUser{api_key} => {
            // check if the User is approved
            let validated = sqlx::query!( r#"
                SELECT id, vouched, last_refresh, event_id from tmp_user WHERE api_key = ?
            "#, api_key).fetch_one(db).await;

            let user_data = match validated {
                Ok(r) => r,
                Err(sqlx::Error::RowNotFound) => return Err(NotFound!("The api_key was not found")),
                Err(_) => return  Err(InternalServer!("Error while fetching user db"))
            };

            if current_timestamp - user_data.last_refresh > 18000 {
                return Err(Forbidden!("Sorry, key was not refreshed"));
            }

            // reset last_refresh
            if let Err(e) = sqlx::query!(r#"
                UPDATE tmp_user SET last_refresh = ? WHERE api_key = ?
            "#, current_timestamp, api_key)
                .execute(db)
                .await {
                    return Err(InternalServer!(format!("There was an Error while Updating the tmp_user ({e})")));

            }

            if !user_data.vouched {
               return Ok(AuthUser::NotApprovedTmpUser { id: user_data.id, api_key });
            }

            // because the user has been vouched for event_id cannot be null
            let event_id = match user_data.event_id {
                Some(id) => id,
                None => return Err(InternalServer!("You don't have a event! Why?"))
            };

            return Ok(AuthUser::TmpUser { id: user_data.id, api_key, event_id })
        },

        RequestUser::Admin{ ref api_key } | RequestUser::AdminWithEvent { ref api_key ,.. } => {
            let user_data_opt= sqlx::query!( r#"
                SELECT last_refresh from user_session WHERE api_key = ?
            "#, api_key).fetch_one(db).await;

            let user_data = match user_data_opt {
                Ok(r) => r,
                Err(sqlx::Error::RowNotFound) => return Err(NotFound!("The admin_api_key was not found")),
                Err(_) => return  Err(InternalServer!("Error while fetching user db"))
            };

            if current_timestamp - user_data.last_refresh > 36000 {
                return Err(Forbidden!("Sorry, key was not refreshed"));
            }

            // reset last_refresh
            if let Err(e) = sqlx::query!(r#"
                UPDATE user_session SET last_refresh = ? WHERE api_key = ?
            "#, current_timestamp, api_key)
                .execute(db)
                .await {
                    return Err(InternalServer!(format!("There was an error reseting the refresh id ({e})")))
            }
            
            // add the event if it exists
            if let RequestUser::AdminWithEvent { event_id, api_key }= user {
                let user_data_opt= sqlx::query!( r#"
                    SELECT id from event WHERE id = ?
                "#, event_id).fetch_one(db).await;

                info!("Eventid: {}", event_id);
                let event_id= match user_data_opt {
                    Ok(r) => r.id,
                    Err(sqlx::Error::RowNotFound) => return Err(NotFound!("The event was not found")),
                    Err(_) => return  Err(InternalServer!("Error while fetching user db"))
                };

                return Ok(AuthUser::AdminWithEvent{ api_key, event_id});
            }

            // Now the only thing that is left is a AdminUser without event
            return Ok(AuthUser::Admin{ api_key: api_key.to_string() });
        }
    }
}

fn req2user(req: &HttpRequest) -> Result<RequestUser, HttpResponse> {

    // check if apikey exists 
    let api_key_opt = req.headers().get(actix_web::http::header::AUTHORIZATION);
    if api_key_opt.is_none() {
        return Err(Unauthorized!("No api_key was supplied"));
    }

    // turn the HeaderValue into a string
    let api_key = match api_key_opt.unwrap().to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return Err(BadRequest!("There where none-ascii characters in the api key"))
    };


    // if the api_key contains a _ it is a Admin key
    return if api_key.contains("_") {

        // check if the event header is set
        let event_opt = req.headers().get("Event");
        if event_opt.is_some() {
            // turn the HeaderValue into a string
            let event_id= match event_opt.unwrap().to_str() {
                Ok(s) => s.to_string(),
                Err(_) => return Err(BadRequest!("There where none-ascii characters in the event_id"))
            };

            return if event_id.trim() == "" {
                Ok(RequestUser::Admin { api_key })
            } else {
                Ok(RequestUser::AdminWithEvent { api_key, event_id})
            }
        }

        Ok(RequestUser::Admin { api_key })

    } else {
        Ok(RequestUser::TmpUser { api_key })
    }
}

pub async fn get_event(event_id: String, db: &SqlitePool) -> Result<Event,HttpResponse> {
    return match sqlx::query_as!(Event, r#"
        SELECT id, name FROM event WHERE id = ?
    "#, event_id).fetch_one(db)
        .await {
            Ok(e) => Ok(e),
            Err(sqlx::Error::RowNotFound) => Err(NotFound!("The event was not found in the auth db")),
            Err(e) => return  Err(InternalServer!(format!("Error while fetching the event ({e})")))
    }
}


/**
 * This function can be used to vouch for a tmp user
 */
pub async fn vouch_tmp_user(db: &SqlitePool, event_id: String, tmp_user_id: String) -> Result<(), HttpResponse> {
    let current_timestamp: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let rows_affected = match sqlx::query!(r#"
        UPDATE tmp_user SET vouched = True, time_of_creation = ?, event_id = ? WHERE id = ?
    "#, current_timestamp, event_id, tmp_user_id)
        .execute(db).await {
            Ok(r) => r.rows_affected(),
            Err(e) => return Err(InternalServer!(format!("There was an error while vouching for user ({e})")))
        };
    
    if rows_affected == 0 {
        return Err(NotFound!("The tmp_user was not found"))
    }

    if rows_affected > 1 {
        return Err(InternalServer!("There are two users by that id. For both has been vouched"))
    }

    return Ok(())
}

/// generate a random hash
fn gen_api_key() -> String {
    let mut rng = rand::thread_rng();
    let mut test: [u8; 64] = [0; 64];
    rand::RngCore::fill_bytes(&mut rng, &mut test);
    let tmp: String = test.into_iter().map(|v| format!("{:x}", v)).collect();
    digest(tmp)
}

/// create a tmp user
pub async fn create_tmp_user(db: &SqlitePool) -> Result<AuthUser, HttpResponse> {
    let current_timestamp: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    //gen Name
    let mut rng = rand::thread_rng();
    let mut name = "".to_string();
    let base32 = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S',
        'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '1', '2', '3', '4', '5', '6', '7',
    ];
    for _ in 1..7 {
        let i = (rng.gen::<f64>() * 32.0) as usize;
        let c = base32.get(i).unwrap() as &char;
        name.push(c.to_owned());
    }

    // gen key
    let key = gen_api_key();

    if sqlx::query!(r#"
        INSERT INTO tmp_user(id, api_key, vouched, time_of_creation, last_refresh) VALUES (?,?,False,?,?)
        "#,
        name,
        key,
        current_timestamp,
        current_timestamp)
        .execute(db)
        .await.is_err() {
            return Err(InternalServer!("There was an error inserting the user into the DB"))
    }

    return Ok(AuthUser::NotApprovedTmpUser {id: name, api_key: key});
}

/** 
 * create a new admin session
 */
pub async fn create_session(db: &SqlitePool) -> Result<AuthUser, HttpResponse> {
    let current_timestamp: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let api_key = format!("TEACH_{}", gen_api_key());

    if let Err(e) = sqlx::query!(r#"
        INSERT INTO user_session(api_key, time_of_creation, last_refresh) VALUES (?,?,?)
    "#, api_key, current_timestamp, current_timestamp)
        .execute(db)
        .await {
            return Err(InternalServer!(format!("Error while inserting into the database ({})", e)))
    }

    return Ok(AuthUser::Admin {api_key})
}
