//! Just a few helper functions for Http Responses

/**
 * Returns a httpResponse with the right message attached 
 *
 * Use it like this:
 * ```rust
 * use emotionLib::NotFound;
 * use actix_web::HttpResponse;
 * NotFound!("Hey this is the message");
 * ```
 * or 
 * ```rust
 * use emotionLib::NotFoundf;
 * use actix_web::HttpResponse;
 * let me = "Hey I am a message";
 * NotFoundf!("Hey this message '{}' was not found", me);
 * ```
 * 
 * Possible are:
 * - Unauthorized
 * - NotFound
 * - Forbidden
 */
#[macro_use]
pub mod res {
    #[macro_export]
    macro_rules! Unauthorized {
        ($message:expr) => {
            HttpResponse::Unauthorized().json(serde_json::json!({"message": $message}))
        };
    }
    #[macro_export]
    macro_rules! NotFound{
        ($message:expr) => {
            HttpResponse::NotFound().json(serde_json::json!({"message": $message}))
        };
    }
    #[macro_export]
    macro_rules! Conflict{
        ($message:expr) => {
            HttpResponse::Conflict().json(serde_json::json!({"message": $message}))
        };
    }
    #[macro_export]
    macro_rules! Forbidden {
        ($message:expr) => {
            HttpResponse::Forbidden().json(serde_json::json!({"message": $message}))
        };
    }
    #[macro_export]
    macro_rules! BadRequest{
        ($message:expr) => {
            HttpResponse::BadRequest().json(serde_json::json!({"message": $message}))
        };
    }
    #[macro_export]
    macro_rules! InternalServer{
        ($message:expr) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"message": $message}))
        };
    }
    // format
    #[macro_export]
    macro_rules! Unauthorizedf {
        ( $( $x:expr ),* ) => {
            HttpResponse::Unauthorized().json(serde_json::json!({"message": format!($($x,)*)}))
        };
    }
    #[macro_export]
    macro_rules! NotFoundf {
        ( $( $x:expr ),* ) => {
            HttpResponse::NotFound().json(serde_json::json!({"message": format!($($x,)*)}))
        };
    }
    #[macro_export]
    macro_rules! Conflictf {
        ( $( $x:expr ),* ) => {
            HttpResponse::Conflict().json(serde_json::json!({"message": format!($($x,)*)}))
        };
    }
    #[macro_export]
    macro_rules! Forbiddenf {
        ( $( $x:expr ),* ) => {
            HttpResponse::Forbidden().json(serde_json::json!({"message": format!($($x,)*)}))
        };
    }
    #[macro_export]
    macro_rules! BadRequestf {
        ( $( $x:expr ),* ) => {
            HttpResponse::BadRequest().json(serde_json::json!({"message": format!($($x,)*)}))
        };
    }
    #[macro_export]
    macro_rules! InternalServerf {
        ( $( $x:expr ),* ) => {
           HttpResponse::InternalServerError().json(serde_json::json!({"message": format!( $( $x, )* )}))
        };
    }
}
