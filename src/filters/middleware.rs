use warp::{cookie, Filter, Rejection};

use crate::{crypto::authn::AuthnToken, errors::ServiceError, models::AuthInfo};

pub fn authn_optional() -> impl Filter<Extract = (AuthInfo,), Error = Rejection> + Copy {
    cookie("token")
        .map(cookie_authn_step2_optional)
        .map(|o: Option<AuthnToken>| {
            o.map(|a| AuthInfo::Authenticated(a.claims.user_id as usize))
                .unwrap_or(AuthInfo::Anonymous)
        })
}

pub fn authn() -> impl Filter<Extract = (usize,), Error = Rejection> + Copy {
    cookie("token")
        .and_then(cookie_authn_step2)
        .map(|token: AuthnToken| token.claims.user_id as usize)
}

async fn cookie_authn_step2(token_str: String) -> Result<AuthnToken, Rejection> {
    let token = AuthnToken::from_str(&token_str).map_err(ServiceError::from)?;
    match token.verify() {
        Ok(_) => Ok(token),
        Err(_) => Err(ServiceError::Unauthorized.into()),
    }
}

fn cookie_authn_step2_optional(token_str: String) -> Option<AuthnToken> {
    let token = AuthnToken::from_str(&token_str).ok()?;
    match token.verify() {
        Ok(_) => Some(token),
        Err(_) => None,
    }
}
