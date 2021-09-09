use std::convert::Infallible;

use warp::{Filter, Rejection, Reply};

use crate::{errors::handle_rejection, filters::helpers::with, handlers, models::Db};

mod helpers;
mod middleware;

pub fn router(db: Db) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    index(db.clone())
        .or(restaurants::router(db.clone()))
        .or(user::router(db))
        .or(static_files::router())
        .recover(handle_rejection)
}

pub fn index(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end().and(with(db)).and_then(handlers::index)
}

mod restaurants {
    use warp::{Filter, Rejection, Reply};

    use crate::{
        filters::{
            helpers::with,
            middleware::{authn, authn_optional},
        },
        handlers,
        models::Db,
    };

    pub fn router(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("restaurants").and(
            list(db.clone())
                .or(detail(db.clone()))
                .or(reviews(db.clone()))
                .or(review(db)),
        )
    }

    fn list(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path::end()
            .and(warp::get())
            .and(with(db))
            .and_then(handlers::list_restaurants)
    }

    fn detail(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!(usize)
            .and(warp::get())
            .and(authn_optional())
            .and(with(db))
            .and_then(handlers::show_restaurant)
    }

    fn reviews(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!(usize / "reviews")
            .and(warp::post())
            .and(authn())
            .and(warp::body::form())
            .and(with(db))
            .and_then(handlers::create_review)
    }

    fn review(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!(usize / "reviews" / usize)
            .and(warp::get())
            .and(with(db))
            .and_then(handlers::show_review)
    }
}

mod user {
    use warp::{Filter, Rejection, Reply};

    use crate::{
        filters::{helpers::with, middleware::authn},
        handlers,
        models::Db,
    };

    pub fn router(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("users").and(
            users(db.clone())
                .or(user(db.clone()))
                .or(register())
                .or(register_action(db.clone()))
                .or(check(db.clone()))
                .or(login())
                .or(login_action(db)),
        )
    }

    fn users(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path::end()
            .and(warp::get())
            .and(with(db))
            .and_then(handlers::show_users)
    }

    fn register_action(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path::end()
            .and(warp::post())
            .and(warp::body::form())
            .and(with(db))
            .and_then(handlers::register_user)
    }

    fn user(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!(usize)
            .and(warp::get())
            .and(with(db))
            .and_then(handlers::profile)
    }

    fn register() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("register")
            .and(warp::get())
            .and_then(handlers::register_user_page)
    }

    fn login() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("login")
            .and(warp::get())
            .and_then(handlers::login_user_page)
    }

    fn login_action(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("login")
            .and(warp::post())
            .and(warp::body::form())
            .and(with(db))
            .and_then(handlers::login_user_action)
    }

    fn check(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("check")
            .and(warp::get())
            .and(authn())
            .and(with(db))
            .and_then(handlers::check)
    }
}

mod static_files {

    use warp::{Filter, Rejection, Reply};

    pub fn router() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        images()
    }

    fn images() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("static").and(warp::fs::dir("./static"))
    }
}
