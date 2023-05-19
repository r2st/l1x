use std::collections::HashMap;
use std::sync::Mutex;
use ed25519_dalek::{VerifyingKey, Signature};
use sha2::{Digest, Sha512};
use primitives::{Address, Balance, MemPoolSize, TimeStamp, Nonce};
use account::account_state::AccountState;
use anyhow::{Error, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};

mod tests;

#[derive(Debug, Clone, PartialEq)]
struct Transaction {
    nonce: Nonce,
    sender: Address,
    recipient: Address,
    amount: Balance,
    gas: Balance,
    fee: Balance,               // Add a fee field to the transaction
    signature: Signature,
    verifying_key: VerifyingKey,
    timestamp: TimeStamp,       // Add a timestamp to each transaction
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
    transactions: Mutex<HashMap<Address, HashMap<TimeStamp, Vec<Transaction>>>>,
    transactions_priority: Mutex<Vec<Transaction>>,  // Use a vector to maintain transactions priority in the mempool
    max_size: MemPoolSize,
    gas_limit: Balance,
    db_path: &'a str,
    rate_limit: usize,              // Number of transactions allowed within a time frame
    time_frame_seconds: TimeStamp,  // Time frame in seconds
    expiration_seconds: TimeStamp,  // Expiration time in seconds
}

impl<'a> Mempool<'a> {
    #[allow(dead_code)]
    fn new(max_size: MemPoolSize, gas_limit: Balance, db_path: &'a str, rate_limit: usize, time_frame_seconds: TimeStamp, expiration_seconds: TimeStamp) -> Self {
        Mempool {
            transactions: Mutex::new(HashMap::new()),
            transactions_priority: Mutex::new(Vec::new()),
            max_size,
            gas_limit,
            db_path,
            rate_limit,
            time_frame_seconds,
            expiration_seconds,
        }
    }
    #[allow(dead_code)]
    fn add_transaction(&self, mut transaction: Transaction) -> Result<(), Error> {
        let mut mempool = self.transactions.lock().unwrap();
        let mut mempool_priority = self.transactions_priority.lock().unwrap();
        
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

        // Retrieve the transactions for the sender from the mempool within the time frame
        let entries: HashMap<TimeStamp, Vec<Transaction>> = match mempool.get(&transaction.sender) {
            Some(sender_transactions) => {
                sender_transactions
                    .clone()
                    .into_iter()
                    .filter(|(timestamp, _)| *timestamp >= (current_time - self.time_frame_seconds))
                    .collect()
            }
            None => HashMap::new(), // No transactions for the sender, so an empty HashMap
        };

        // Calculate the sender's transaction count
        let sender_transaction_count = entries.values().flatten().count();

        if sender_transaction_count >= self.rate_limit {
            return Err(anyhow!("Rate limit exceeded for the sender"));
        }

        // Check for conflicts with existing transactions
        let mut conflicts: Vec<Transaction> = Vec::new();

        mempool_priority.retain(|existing_tx| {
            if self.has_conflict(existing_tx, &transaction) {
                conflicts.push(existing_tx.clone());
                false // Remove the conflicting transaction from the mempool
            } else {
                true // Keep the non-conflicting transaction in the mempool
            }
        });

        if !conflicts.is_empty() {
            // Remove conflicting transactions from mempool
            for conflict in &conflicts {
                let sender_transactions = mempool.get_mut(&conflict.sender);
                if let Some(sender_transactions) = sender_transactions {
                    let timestamp = &conflict.timestamp;
                    if let Some(transactions_by_timestamp) = sender_transactions.get_mut(timestamp) {
                        transactions_by_timestamp.retain(|tx| tx.nonce != conflict.nonce);
                    }
                }
            }
            // Handle conflicts based on transaction fees
            transaction = self.handle_conflicts(conflicts, transaction);
        }
        transaction.timestamp = current_time;
        let entry = mempool
            .entry(transaction.sender.clone())
            .or_insert_with(HashMap::new)
            .entry(current_time)
            .or_insert_with(Vec::new);
        entry.push(transaction.clone());

        // Determine the insertion position based on fee (higher fee first)
        let insertion_pos = mempool_priority
            .iter()
            .position(|tx| tx.fee < transaction.fee)
            .unwrap_or_else(|| mempool.len()-1);

        // Insert the transaction at the determined position
        mempool_priority.insert(insertion_pos, transaction);
        Ok(())
    }
    #[allow(dead_code)]
    fn has_conflict(&self, existing_tx: &Transaction, new_tx: &Transaction) -> bool {
        existing_tx.sender == new_tx.sender && existing_tx.nonce == new_tx.nonce
    }
    #[allow(dead_code)]
    fn handle_conflicts(&self, conflicts: Vec<Transaction>, new_tx: Transaction) -> Transaction {        
        let mut transaction = new_tx.clone();
        for conflict in conflicts {
            if new_tx.fee < conflict.fee {
                transaction = conflict.clone()
            }
        }
        transaction
    }
    #[allow(dead_code)]
    fn remove_transaction(&self, sender: &Address) {
        let mut mempool = self.transactions.lock().unwrap();
        mempool.remove(sender);
    }
    #[allow(dead_code)]
    fn get_transactions(&self) -> Vec<Transaction> {
        let mempool = self.transactions.lock().unwrap();
        mempool.values().flat_map(|transactions_by_timestamp| {
            transactions_by_timestamp
                .values()
                .flatten()
                .cloned()
        }).collect()
    }
    #[allow(dead_code)]
    fn get_transactions_priority(&self) -> Vec<Transaction> {
        let mempool = self.transactions_priority.lock().unwrap();
        mempool.clone()
    }
    #[allow(dead_code)]
    fn get_transactions_by_address(&self, address: Address) -> Vec<Transaction> {
        let mempool = self.transactions.lock().unwrap();
        if let Some(transactions_by_timestamp) = mempool.get(&address) {
            transactions_by_timestamp
                .values()
                .flatten()
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    #[allow(dead_code)]
    fn remove_expired_transactions(&self) {
        let mut mempool = self.transactions.lock().unwrap();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        mempool.retain(|_sender, transactions_by_timestamp| {
            transactions_by_timestamp.retain(|timestamp, _transactions| {
                *timestamp >= current_time - self.expiration_seconds
            });
            !transactions_by_timestamp.is_empty()
        });
    }
}
