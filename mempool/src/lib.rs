use std::collections::{HashMap};
use std::sync::Mutex;
use ed25519_dalek::{VerifyingKey, Signature};
use sha2::{Digest, Sha512};
use primitives::{Address, Balance, MemPoolSize};
use account::account_state::AccountState;
use anyhow::{Error, anyhow};

mod tests;

#[derive(Debug, Clone, PartialEq)]
struct Transaction {
    sender: Address,
    recipient: Address,
    amount: Balance,
    gas: Balance,
    signature: Signature,
    verifying_key: VerifyingKey,
}

impl Transaction {
    #[allow(dead_code)]
    fn verify_signature(&self) -> bool {
        let mut hasher = Sha512::new();
        hasher.update(&self.sender);
        hasher.update(&self.recipient);
        hasher.update(&self.amount.to_be_bytes());
        let message = hasher.finalize();
        self.verifying_key.verify_strict(&message, &self.signature).is_ok()
    }
}

struct Mempool<'a> {
    transactions: Mutex<HashMap<Address, Transaction>>,
    max_size: MemPoolSize,
    gas_limit: Balance,
    db_path: &'a str,
}

impl<'a> Mempool<'a> {
    #[allow(dead_code)]
    fn new(max_size: MemPoolSize, gas_limit: Balance, db_path: &'a str) -> Self {
        Mempool {
            transactions: Mutex::new(HashMap::new()),
            max_size,
            gas_limit,
            db_path,
        }
    }
    #[allow(dead_code)]
    fn add_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        let mut mempool = self.transactions.lock().unwrap();

        // Check if the mempool is full
        if mempool.len() >= self.max_size {
            return Err(anyhow!("Mempool is full"));
        }

        // Check if the transaction gas is within the gas limit
        if transaction.gas > self.gas_limit {
            return Err(anyhow!("Transaction gas exceeds gas limit"));
        }

        // Check if the transaction signature is valid
        if !transaction.verify_signature() {
            return Err(anyhow!("Invalid transaction signature"));
        }

        // Create an instance of AccountState and fetch the account from the database
        let account_state = AccountState::new(&self.db_path).unwrap();

        // Check if the transaction sender is valid
        if !account_state.is_valid_account(transaction.sender.clone()) {
            return Err(anyhow!("Invalid transaction sender"));
        }

        let account = account_state.get_account(transaction.sender.clone()).unwrap();

        // Check if the account has sufficient balance for gas
        if account.balance < transaction.gas {
            return Err(anyhow!("Insufficient balance to cover gas"));
        }

        // Add the transaction to the mempool
        mempool.insert(transaction.sender.clone(), transaction);

        Ok(())
    }
    #[allow(dead_code)]
    fn remove_transaction(&self, sender: &Address) {
        let mut mempool = self.transactions.lock().unwrap();
        mempool.remove(sender);
    }
    #[allow(dead_code)]
    fn get_transactions(&self) -> Vec<Transaction> {
        let mempool = self.transactions.lock().unwrap();
        mempool.values().cloned().collect()
    }
}
