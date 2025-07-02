use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn ensure_event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    ensure_event_impl(item)
}

fn ensure_event_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let ItemFn {
        sig,
        vis,
        block,
        attrs,
    } = input;

    let statements = block.stmts;

    // Reconstruct the function as output using parsed input
    quote!(
        // Reapply all the other attributes on this function.
        // The compiler doesn't include the macro we are
        
        #(#attrs)*
        #vis #sig {
            let __user = match emotionLib::auth::get_user(&req, &data.db).await {
                Ok(u) => u,
                Err(e) => return e
            };
            let __event_id = match __user {
                AuthUser::TmpUser {event_id, ..} => event_id,
                AuthUser::AdminWithEvent {event_id, ..} => event_id,
                AuthUser::Admin {..} | AuthUser::NotApprovedTmpUser {..} => return HttpResponse::Forbidden().into()
            };

            let event = match emotionLib::auth::get_event(__event_id.to_string(), &data.db).await {
                Ok(e) => e,
                Err(e) => return e
            };

            #(#statements)*
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn ensure_user(_attr: TokenStream, item: TokenStream) -> TokenStream {
    ensure_user_impl(item)
}

fn ensure_user_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let ItemFn {
        sig,
        vis,
        block,
        attrs,
    } = input;

    let statements = block.stmts;

    // Reconstruct the function as output using parsed input
    quote!(
        // Reapply all the other attributes on this function.
        // The compiler doesn't include the macro we are
        
        #(#attrs)*
        #vis #sig {
            let user = match emotionLib::auth::get_user(&req, &data.db).await {
                Ok(u) => u,
                Err(e) => return e
            };

            #(#statements)*
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn ensure_admin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    ensure_admin_impl(item)
}

fn ensure_admin_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let ItemFn {
        sig,
        vis,
        block,
        attrs,
    } = input;

    let statements = block.stmts;

    // Reconstruct the function as output using parsed input
    quote!(
        // Reapply all the other attributes on this function.
        // The compiler doesn't include the macro we are
        
        #(#attrs)*
        #vis #sig {
            let user = match emotionLib::auth::get_user(&req, &data.db).await {
                Ok(u) => u,
                Err(e) => return e
            };

            if(matches!(user, AuthUser::TmpUser{..}) || matches!(user, AuthUser::NotApprovedTmpUser{..})) {
                return HttpResponse::Forbidden().into(); 
            }

            #(#statements)*
        }
    )
    .into()
}

