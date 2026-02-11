// Minimal, Stylus‑style Rust contract stub for demo purposes.
//
// This is NOT a full Stylus contract; it exists to give the debug engine,
// test runtime, and migration flow a concrete file to point at.

pub struct Vault {
    balance: u128,
}

impl Vault {
    pub fn new() -> Self {
        Self { balance: 0 }
    }

    pub fn deposit(&mut self, amount: u128) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: u128) {
        self.balance -= amount;
    }

    pub fn deposit_and_withdraw(&mut self, amount: u128) {
        self.deposit(amount);
        self.withdraw(amount / 2);
    }
}


