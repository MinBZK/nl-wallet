use std::fmt::Debug;

pub struct WalletLock {
    is_locked: bool,
    update_callback: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl WalletLock {
    pub fn new(is_locked: bool) -> Self {
        WalletLock {
            is_locked,
            update_callback: None,
        }
    }

    fn call_update_callback(&self) {
        if let Some(update_callback) = &self.update_callback {
            update_callback(self.is_locked)
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn lock(&mut self) {
        if self.is_locked {
            return;
        }

        self.is_locked = true;
        self.call_update_callback();
    }

    pub fn unlock(&mut self) {
        if !self.is_locked {
            return;
        }

        self.is_locked = false;
        self.call_update_callback();
    }

    pub fn set_lock_callback<F>(&mut self, callback: F)
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.update_callback.replace(Box::new(callback));
    }

    pub fn clear_lock_callback(&mut self) {
        self.update_callback.take();
    }
}

impl Debug for WalletLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletLock")
            .field("is_locked", &self.is_locked)
            .finish()
    }
}
