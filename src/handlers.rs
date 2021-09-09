use std::{convert::Infallible, str::FromStr};

use askama_warp::Template;
use warp::{
    hyper::{Response, Uri},
    Rejection, Reply,
};

use crate::{
    crypto::{authn::AuthnToken, pwhash},
    errors::ServiceError,
    models::{AuthInfo, CreateReview, Db, Rating, UserPassword},
};

pub async fn index(db: Db) -> Result<impl Reply, Infallible> {
    #[derive(Template)]
    #[template(path = "index.html")]
    struct IndexTemplate {
        restaurants: Vec<RestaurantDisplay>,
    }

    struct RestaurantDisplay {
        id: usize,
        name: String,
        review_summary: ReviewSummary,
    }

    struct ReviewSummary {
        count: usize,
        average: f32,
    }

    let world = db.lock().await;

    let restaurants = world
        .all_restaurants()
        .into_iter()
        .map(|r| RestaurantDisplay {
            id: r.id,
            name: r.name,
            review_summary: {
                let reviews = world.find_reviews_by_restaurant(r.id);
                let count = reviews.len();
                let average = reviews.into_iter().map(|r| r.rating.0).sum::<f32>() / count as f32;
                ReviewSummary { count, average }
            },
        })
        .collect();

    Ok(IndexTemplate { restaurants })
}

pub async fn list_restaurants(db: Db) -> Result<impl Reply, Infallible> {
    #[derive(Template)]
    #[template(path = "restaurants/list.html")]
    struct RestaurantsTemplate {
        restaurants: Vec<RestaurantDisplay>,
    }

    struct RestaurantDisplay {
        id: usize,
        name: String,
        review_summary: ReviewSummary,
    }

    struct ReviewSummary {
        count: usize,
        average: f32,
    }

    let world = db.lock().await;

    let restaurants = world
        .all_restaurants()
        .into_iter()
        .map(|r| RestaurantDisplay {
            id: r.id,
            name: r.name,
            review_summary: {
                let reviews = world.find_reviews_by_restaurant(r.id);
                let count = reviews.len();
                let average = reviews.into_iter().map(|r| r.rating.0).sum::<f32>() / count as f32;
                ReviewSummary { count, average }
            },
        })
        .collect();

    Ok(RestaurantsTemplate { restaurants })
}

pub async fn show_restaurant(id: usize, auth: AuthInfo, db: Db) -> Result<impl Reply, Rejection> {
    #[derive(Template)]
    #[template(path = "restaurants/detail.html")]
    struct RestaurantTemplate {
        id: usize,
        name: String,
        reviews: Vec<ReviewDisplay>,
        auth_info: AuthInfo,
    }

    struct ReviewDisplay {
        id: usize,
        comment: String,
        rating: f32,
        user: UserDisplay,
    }

    struct UserDisplay {
        id: usize,
        name: String,
    }

    let world = db.lock().await;

    let restaurant = world
        .find_restaurant_by_id(id)
        .ok_or_else(|| Rejection::from(ServiceError::NotFound))?;

    let reviews = world
        .find_reviews_by_restaurant(id)
        .into_iter()
        .map(|r| ReviewDisplay {
            id: r.id,
            comment: r.comment,
            rating: r.rating.0,
            user: {
                let user = world.find_user(r.writer).expect("Assume no ghost reviews");
                UserDisplay {
                    id: user.id,
                    name: user.name,
                }
            },
        })
        .collect();

    Ok(RestaurantTemplate {
        id: restaurant.id,
        name: restaurant.name,
        auth_info: auth,
        reviews,
    })
}

pub async fn create_review(
    restaurant_id: usize,
    token: AuthnToken,
    review: CreateReview,
    db: Db,
) -> Result<impl Reply, Rejection> {
    let mut world = db.lock().await;

    let user_id = token.claims.user_id as usize;

    let review = world.create_review(
        review.review,
        Rating::new(review.rating)?,
        restaurant_id,
        user_id,
        None,
    );

    Ok(warp::redirect::see_other(
        Uri::from_str(&format!(
            "/restaurants/{}/reviews/{}",
            restaurant_id, review
        ))
        .unwrap(),
    ))
}

pub async fn show_review(
    restaurant_id: usize,
    review_id: usize,
    db: Db,
) -> Result<impl Reply, Rejection> {
    #[derive(Template)]
    #[template(path = "restaurants/review.html")]
    struct ShowReviewTemplate {
        review: String,
        user: UserDisplay,
        image_path: Option<String>,
        restaurant: RestaurantDisplay,
    }

    struct UserDisplay {
        id: usize,
        name: String,
    }

    struct RestaurantDisplay {
        id: usize,
        name: String,
    }

    let world = db.lock().await;

    let review = world
        .find_reviews_by_restaurant(restaurant_id)
        .into_iter()
        .find(|r| r.id == review_id)
        .ok_or(ServiceError::NotFound)?;

    let restaurant = world
        .find_restaurant_by_id(restaurant_id)
        .ok_or(ServiceError::NotFound)?;

    let user = world
        .find_user(review.writer)
        .expect("Assuming no ghost reviews");

    Ok(ShowReviewTemplate {
        review: review.comment,
        image_path: None,
        user: UserDisplay {
            id: user.id,
            name: user.name,
        },
        restaurant: RestaurantDisplay {
            id: restaurant.id,
            name: restaurant.name,
        },
    })
}

pub async fn show_users(db: Db) -> Result<impl Reply, Rejection> {
    #[derive(Template)]
    #[template(path = "user/list.html")]
    struct UserListTemplate {
        users: Vec<UserDisplay>,
    }

    struct UserDisplay {
        id: usize,
        name: String,
    }

    let world = db.lock().await;
    let users = world
        .get_users()
        .into_iter()
        .map(|u| UserDisplay {
            name: u.name,
            id: u.id,
        })
        .collect();

    Ok(UserListTemplate { users })
}

pub async fn register_user_page() -> Result<impl Reply, Rejection> {
    #[derive(Template)]
    #[template(path = "user/register.html")]
    struct RegisterTemplate {}

    Ok(RegisterTemplate {})
}

pub async fn register_user(user: UserPassword, db: Db) -> Result<impl Reply, Rejection> {
    let user_id = {
        let mut world = db.lock().await;

        if world.find_user_by_name(&user.username).is_some() {
            return Err(Rejection::from(ServiceError::AlreadyExists));
        }

        let pass_hash = pwhash::hash_password(&user.password)?;
        world.create_user(user.username, pass_hash)
    };

    let token = AuthnToken::from_user_id(user_id as i64)?;
    Response::builder().header("Set-Cookie", token.header_val());

    // Post/Redirect/Get pattern
    Ok(warp::reply::with_header(
        warp::redirect::see_other(
            Uri::from_str(&format!("/users/{}", user_id)).expect("This is known to be well-formed"),
        ),
        "Set-Cookie",
        token.header_val(),
    ))
}

pub async fn profile(user: usize, db: Db) -> Result<impl Reply, Rejection> {
    #[derive(Template)]
    #[template(path = "user/profile.html")]
    struct ProfileTemplate {
        name: String,
        reviews: Vec<ReviewDisplay>,
    }

    struct ReviewDisplay {
        id: usize,
        comment: String,
        rating: f32,
        restaurant: RestaurantDisplay,
    }

    struct RestaurantDisplay {
        id: usize,
        name: String,
    }

    let world = db.lock().await;
    let user = world
        .find_user(user)
        .ok_or_else(|| Rejection::from(ServiceError::NotFound))?;

    let reviews_by_user = world.find_reviews_by_user(user.id);

    Ok(ProfileTemplate {
        name: user.name,
        reviews: reviews_by_user
            .into_iter()
            .map(|r| ReviewDisplay {
                id: r.id,
                comment: r.comment,
                rating: r.rating.0,
                restaurant: {
                    let restaurant = world
                        .find_restaurant_by_id(r.restaurant)
                        .expect("Assume no ghost reviews");
                    RestaurantDisplay {
                        id: restaurant.id,
                        name: restaurant.name,
                    }
                },
            })
            .collect(),
    })
}

pub async fn check(token: AuthnToken, db: Db) -> Result<impl Reply, Rejection> {
    #[derive(Template)]
    #[template(path = "user/check.html")]
    struct ProfileTemplate {
        name: String,
    }

    let user_id = token.claims.user_id as usize;

    let world = db.lock().await;
    let user = world
        .find_user(user_id)
        .ok_or_else(|| Rejection::from(ServiceError::NotFound))?;

    Ok(ProfileTemplate { name: user.name })
}

pub async fn login_user_page() -> Result<impl Reply, Infallible> {
    #[derive(Template)]
    #[template(path = "user/login.html")]
    struct LoginTemplate {}

    Ok(LoginTemplate {})
}

pub async fn login_user_action(incoming: UserPassword, db: Db) -> Result<impl Reply, Rejection> {
    let world = db.lock().await;

    let user = match world.find_user_by_name(&incoming.username) {
        Some(user) => user,
        None => return Err(ServiceError::NotFound.into()),
    };

    pwhash::verify(&user.hash, &incoming.password)?;

    Ok(warp::redirect::see_other(
        Uri::from_str(&format!("/users/{}", user.id)).unwrap(),
    ))
}
