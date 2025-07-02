//! Just a few helper functions for Http Responses

/// retruns the given httpBuilder with the message as json in the body
macro_rules! buildRes {
    ($res_builder:expr, $message:expr) => {
        $res_builder.json(json!({"message": $message}))
    };
}

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
            Err(buildRes!(HttpResponse::Unauthorized(), format!("{}", $message)))
        };
    }
    #[macro_export]
    macro_rules! NotFound{
        ($message:expr) => {
            Err(buildRes!(HttpResponse::NotFound(), format!("{}", $message)))
        };
    }
    #[macro_export]
    macro_rules! Forbidden {
        ($message:expr) => {
            Err(buildRes!(HttpResponse::Forbidden(), format!("{}", $message)))
        };
    }
    #[macro_export]
    macro_rules! BadRequest{
        ($message:expr) => {
            Err(buildRes!(HttpResponse::BadRequest(), format!("{}", $message)))
        };
    }
    #[macro_export]
    macro_rules! InternalServer{
        ($message:expr) => {
            Err(buildRes!(HttpResponse::InternalServerError(), format!("{}", $message)))
        };
    }
}
