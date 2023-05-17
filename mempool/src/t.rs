use std::collections::HashMap;
use std::sync::Mutex;
use ed25519_dalek::{VerifyingKey, Signature};
use sha2::{Digest, Sha512};
use primitives::{Address, Balance, MemPoolSize};
use account::account_state::AccountState;
use anyhow::{Error, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};

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
    transactions: Mutex<HashMap<Address, (Transaction, u64)>>, // Add a timestamp to each transaction
    max_size: MemPoolSize,
    gas_limit: Balance,
    db_path: &'a str,
    rate_limit: u64,       // Number of transactions allowed within a time frame
    time_frame_seconds: u64,  // Time frame in seconds
}

impl<'a> Mempool<'a> {
    #[allow(dead_code)]
    fn new(max_size: MemPoolSize, gas_limit: Balance, db_path: &'a str, rate_limit: u64, time_frame_seconds: u64) -> Self {
        Mempool {
            transactions: Mutex::new(HashMap::new()),
            max_size,
            gas_limit,
            db_path,
            rate_limit,
            time_frame_seconds,
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

        // Check if the account has sufficient balance for the transaction amount and gas
        let total_cost = transaction.amount + transaction.gas;
        if account.balance < total_cost {
            return Err(anyhow!("Insufficient balance to cover the transaction"));
        }

        // Check if the sender has exceeded the rate limit within the time frame
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let sender_transactions = mempool.entry(transaction.sender.clone()).or_insert((transaction, current_time));
        let sender_transaction_count = mempool.values().filter(|(tx, timestamp)| {
            *timestamp >= &(current_time - self.time_frame_seconds) && tx.sender == transaction.sender
        }).count();

        if sender_transaction_count >= self.rate_limit {
            return Err(anyhow!("Rate limit exceeded for the sender"));
        }

        // Update the transaction timestamp for the sender
        sender_transactions.1 = current_time;

        // Add the transaction to the mempool
        mempool.insert(transaction.sender.clone(), (transaction, current_time));

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
        mempool.values().map(|(tx, _)| tx.clone()).collect()
    }
}
