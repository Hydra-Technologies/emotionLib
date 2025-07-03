//! Just a few helper functions for Http Responses

/**
 * Returns a httpResponse with the right message attached 
 *
 * Use it like this:
 * ```rust
 *  httpRes!(Unauthorized: "Hey this is the message")
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
}
