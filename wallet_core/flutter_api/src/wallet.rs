use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use wallet::{init_wallet, Wallet};

pub static WALLET: Lazy<Mutex<Wallet>> = Lazy::new(|| Mutex::new(init_wallet()));
