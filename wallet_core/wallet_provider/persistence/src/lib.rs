pub mod database;
pub mod entity;
pub mod postgres;
pub mod repositories;
pub mod transaction;
pub mod wallet_user_repository;

pub trait PersistenceConnection<T> {
    fn connection(&self) -> &T;
}
