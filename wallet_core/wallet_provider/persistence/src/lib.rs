pub mod database;
#[rustfmt::skip]
pub mod entity;
pub mod repositories;
pub mod transaction;
pub mod wallet_user;
pub mod wallet_user_key;

pub trait PersistenceConnection<T> {
    fn connection(&self) -> &T;
}
