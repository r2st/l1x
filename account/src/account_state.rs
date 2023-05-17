use primitives::{Address, Balance};
use rocksdb::{DB, Options, WriteBatch};
use crate::account::Account;
use anyhow::{Error, anyhow};

pub struct AccountState {
    db: DB,
}

impl AccountState {
    #[allow(dead_code)]
    pub fn new(db_path: &str) -> Result<Self, Error> {
        let mut options = Options::default();
        options.create_if_missing(true);
        let db = match DB::open(&options, db_path) {
            Ok(db) => db,
            Err(_e) => return Err(anyhow!("Failed to open database")),
        };

        Ok(AccountState { db })
    }
    #[allow(dead_code)]
    pub fn get_account(&self, address: Address) -> Result<Account, Error> {
        let balance = match self.db.get(address) {
            Ok(Some(value)) => {
                let balance_bytes = value.as_slice();
                let balance: Balance = match bincode::deserialize(balance_bytes) {
                    Ok(b) => b,
                    Err(_e) => return Err(anyhow!("Failed to fetch balance")),
                };
                balance
            }
            _ => 0,
        };

        Ok(Account {
            address,
            balance,
        })
    }
    #[allow(dead_code)]
    pub fn update_account(&self, account: &Account) -> Result<(), Error> {
        let mut batch = WriteBatch::default();
        let balance_bytes = match bincode::serialize(&account.balance) {
            Ok(b) => b,
            Err(_e) => return Err(anyhow!("Failed to update balance")),
        };
        batch.put(account.address, balance_bytes);
        self.db.write(batch)?;

        Ok(())
    }
    #[allow(dead_code)]
    pub fn is_valid_account(&self, address: Address) -> bool {
        match self.db.get(address).unwrap() {
            Some(_) => true,
            None => false,
        }
    }
}
