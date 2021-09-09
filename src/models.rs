use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::Mutex;

use crate::errors::ServiceError;

pub type Db = Arc<Mutex<World>>;

#[derive(Clone)]
pub struct Restaurant {
    pub id: usize,
    pub name: String,
    pub description: String,
}

/// Real value in the [0; 5] range
#[derive(Clone, Copy)]
pub struct Rating(pub f32);

impl Rating {
    pub fn new(rating: f32) -> Result<Rating, ServiceError> {
        if !(0.0..=5.0).contains(&rating) {
            Err(ServiceError::RatingNotInRange(rating))
        } else {
            Ok(Rating(rating))
        }
    }
}

/// Review of restaurant
#[derive(Clone)]
pub struct Review {
    pub id: usize,
    pub comment: String,
    pub rating: Rating,
    pub restaurant: usize,
    pub writer: usize,
    pub image_name: Option<String>,
}

#[derive(Clone)]
pub struct User {
    pub id: usize,
    pub name: String,
    pub hash: String,
}

#[derive(Default)]
pub struct World {
    restaurants: Vec<Restaurant>,
    reviews: Vec<Review>,
    users: Vec<User>,
}

impl World {
    pub fn create_restaurant(&mut self, name: String, description: String) -> usize {
        let id = self.restaurants.len();
        self.restaurants.push(Restaurant {
            id,
            name,
            description,
        });
        id
    }

    pub fn create_review(
        &mut self,
        comment: String,
        rating: Rating,
        restaurant: usize,
        writer: usize,
        image_name: Option<String>,
    ) -> usize {
        let id = self.reviews.len();
        self.reviews.push(Review {
            id,
            comment,
            rating,
            restaurant,
            writer,
            image_name,
        });
        id
    }

    /// Should encapsulate hashing into this function to avoid accidental bypass
    pub fn create_user(&mut self, username: String, hash: String) -> usize {
        let id = self.users.len();
        self.users.push(User {
            id,
            name: username,
            hash,
        });
        id
    }

    pub fn all_restaurants(&self) -> Vec<Restaurant> {
        self.restaurants.clone()
    }

    pub fn find_restaurant_by_id(&self, id: usize) -> Option<Restaurant> {
        self.restaurants.get(id).cloned()
    }

    pub fn find_reviews_by_restaurant(&self, restaurant: usize) -> Vec<Review> {
        self.reviews
            .iter()
            .filter(|r| r.restaurant == restaurant)
            .cloned()
            .collect()
    }

    pub fn find_reviews_by_user(&self, user_id: usize) -> Vec<Review> {
        self.reviews
            .iter()
            .filter(|r| r.writer == user_id)
            .cloned()
            .collect()
    }

    pub fn get_users(&self) -> Vec<User> {
        self.users.clone()
    }

    pub fn find_user(&self, id: usize) -> Option<User> {
        self.users.get(id).cloned()
    }

    pub fn find_user_by_name(&self, name: &str) -> Option<User> {
        self.users.iter().find(|u| u.name == name).cloned()
    }
}

#[derive(Deserialize)]
pub struct UserPassword {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateReview {
    pub review: String,
    pub rating: f32,
}

pub enum AuthInfo {
    Authenticated(usize),
    Anonymous,
}
