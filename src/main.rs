use std::{env, sync::Arc};

use tokio::sync::Mutex;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

use crate::models::{Rating, World};

mod crypto;
mod errors;
mod filters;
mod handlers;
mod models;

fn world() -> World {
    let mut world = World::default();

    let bonnie = world.create_user("Bonnie".to_string(), "bar".to_string());

    world.create_user("Annie".to_string(), "bar".to_string());

    let bennys = world.create_restaurant(
        "Benny's Burger Bar".to_string(),
        "Benny Belches Bountiful Burgers By The Billions".to_string(),
    );
    world.create_review(
        "Avoid at all costs".to_string(),
        Rating::new(0.0).unwrap(),
        bennys,
        bonnie,
        Some("cat.jpg".to_string()),
    );

    let _sallys = world.create_restaurant(
        "Sally's Savory Sautés".to_string(),
        "Sally Seeks Sanitary Sambuca Shots".to_string(),
    );

    let _docs = world.create_restaurant(
        "Doc's Diner".to_string(),
        "Doc Defines Dogday Dinners".to_string(),
    );

    world
}

#[tokio::main]
async fn main() {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_string());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let world = world();
    let db = Arc::new(Mutex::new(world));

    let filter = filters::router(db).with(warp::trace::request());

    warp::serve(filter).run(([127, 0, 0, 1], 3030)).await;
}
