#[cfg(test)]
mod tests {
    use crate::*;
    use ed25519_dalek::{SigningKey, SECRET_KEY_LENGTH, Signer};
    use tempfile::TempDir;

    fn sign(sender: Address, recipient: Address, amount: Balance, signing_key: &SigningKey) -> Signature {
        let mut hasher = Sha512::new();
        hasher.update(sender);
        hasher.update(recipient);
        hasher.update(amount.to_be_bytes());
        let message = hasher.finalize();
        signing_key.sign(&message)
    }

    fn create_account(account_address: Address, account_balance: Balance, db_path: &str) {
        let account_state = AccountState::new(db_path).unwrap();
        let mut account = account_state
            .get_account(account_address)
            .unwrap();

        account.balance = account_balance;
        account_state.update_account(&account).unwrap();

        let retrieved_account = account_state
            .get_account(account_address)
            .unwrap();

        assert_eq!(retrieved_account.address, account_address);
        assert_eq!(retrieved_account.balance, account_balance);
    }

    #[test]
    fn test_mempool_add_transaction() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, 60);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        create_account(sender, 5000, db_path);
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert!(mempool.add_transaction(transaction).is_ok());
    }

    #[test]
    fn test_mempool_add_transaction_invalid_sender() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, 60);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert_eq!(
            mempool.add_transaction(transaction).err().unwrap().to_string(),
            "Invalid transaction sender"
        );
    }

    #[test]
    fn test_mempool_add_transaction_full() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(1, 1000, db_path, 10, 60, 60);
        let sender1: Address = [1u8; 32].into();
        let recipient1: Address = [2u8; 32].into();
        let sec_bytes1 = &[1u8; SECRET_KEY_LENGTH];
        let signing_key1 = SigningKey::from_bytes(sec_bytes1);
        create_account(sender1, 5000, db_path);
        let transaction1 = Transaction {
            sender: sender1.clone(),
            recipient: recipient1.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender1.clone(), recipient1.clone(), 10, &signing_key1),
            verifying_key: signing_key1.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        let sender2: Address = [1u8; 32].into();
        let recipient2: Address = [2u8; 32].into();
        let sec_bytes2 = &[1u8; SECRET_KEY_LENGTH];
        let signing_key2 = SigningKey::from_bytes(sec_bytes2);
        create_account(sender2, 5000, db_path);
        let transaction2 = Transaction {
            sender: sender2.clone(),
            recipient: recipient2.clone(),
            amount: 5,
            gas: 50,
            fee: 100,
            signature: sign(sender2.clone(), recipient2.clone(), 10, &signing_key2),
            verifying_key: signing_key2.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        assert!(mempool.add_transaction(transaction1.clone()).is_ok());
        assert_eq!(
            mempool.add_transaction(transaction2.clone()).err().unwrap().to_string(),
            "Mempool is full"
        );
    }

    #[test]
    fn test_mempool_add_transaction_gas_exceeds_limit() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, 60);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        create_account(sender, 5000, db_path);
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 2000, // exceeds gas limit
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert_eq!(
            mempool.add_transaction(transaction).err().unwrap().to_string(),
            "Transaction gas exceeds gas limit"
        );
    }

    #[test]
    fn test_mempool_add_transaction_invalid_signature() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, 60);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        let sec_bytes1 = &[2u8; SECRET_KEY_LENGTH];
        let signing_key1 = SigningKey::from_bytes(sec_bytes1);
        create_account(sender, 5000, db_path);
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key1),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert_eq!(
            mempool.add_transaction(transaction).err().unwrap().to_string(),
            "Invalid transaction signature"
        );
    }

    #[test]
    fn test_insufficient_balance() {
        // Create a transaction with an amount greater than the account balance
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, 60);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        create_account(sender, 5000, db_path);
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10000,
            gas: 100,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10000, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert_eq!(
            mempool.add_transaction(transaction).err().unwrap().to_string(),
            "Insufficient balance to cover the transaction"
        );
    }

    #[test]
    fn test_mempool_add_transaction_rate_limit_exceeded() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 2, 10, 60); // Set rate limit to 2 transactions within 10 seconds

        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        create_account(sender, 5000, db_path);

        let transaction1 = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        let transaction2 = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 5,
            gas: 50,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 5, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        let transaction3 = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 3,
            gas: 30,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 3, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        assert!(mempool.add_transaction(transaction1.clone()).is_ok());
        assert!(mempool.add_transaction(transaction2.clone()).is_ok());
        assert_eq!(
            mempool.add_transaction(transaction3.clone()).err().unwrap().to_string(),
            "Rate limit exceeded for the sender"
        );
    }

    #[test]
    fn test_mempool_remove_transaction() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, 60);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        create_account(sender, 5000, db_path);
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert!(mempool.add_transaction(transaction).is_ok());
        mempool.remove_transaction(&sender);
        let transactions = mempool.get_transactions();
        assert!(transactions.is_empty());
    }

    #[test]
    fn test_mempool_get_transactions() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, 60);
        let sender1: Address = [1u8; 32].into();
        let recipient1: Address = [2u8; 32].into();
        let sec_bytes1 = &[1u8; SECRET_KEY_LENGTH];
        let signing_key1 = SigningKey::from_bytes(sec_bytes1);
        create_account(sender1, 5000, db_path);
        let transaction1 = Transaction {
            sender: sender1.clone(),
            recipient: recipient1.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender1.clone(), recipient1.clone(), 10, &signing_key1),
            verifying_key: signing_key1.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        let sender2: Address = [3u8; 32].into();
        let recipient2: Address = [4u8; 32].into();
        let sec_bytes2 = &[5u8; SECRET_KEY_LENGTH];
        let signing_key2 = SigningKey::from_bytes(sec_bytes2);
        create_account(sender2, 5000, db_path);
        let transaction2 = Transaction {
            sender: sender2.clone(),
            recipient: recipient2.clone(),
            amount: 5,
            gas: 50,
            fee: 100,
            signature: sign(sender2.clone(), recipient2.clone(), 5, &signing_key2),
            verifying_key: signing_key2.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert!(mempool.add_transaction(transaction1.clone()).is_ok());
        assert!(mempool.add_transaction(transaction2.clone()).is_ok());
        let transactions = mempool.get_transactions();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.contains(&transaction1));
        assert!(transactions.contains(&transaction2));
    }

    #[test]
    fn test_mempool_remove_expired_transactions() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let expiration_seconds: TimeStamp = 2; // Set expiration time to 2 seconds

        // Create a new mempool with expiration time and rate limit
        let mempool = Mempool::new(100, 1000, db_path, 10, 60, expiration_seconds);

        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        create_account(sender, 5000, db_path);
        // Add a transaction to the mempool
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 100,
            fee: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        assert!(mempool.add_transaction(transaction.clone()).is_ok());

        // Wait for the expiration time to pass
        std::thread::sleep(std::time::Duration::from_secs(expiration_seconds + 1));

        // Call remove_expired_transactions
        mempool.remove_expired_transactions();

        // Verify that the transaction is removed from the mempool
        assert!(mempool.get_transactions().is_empty());
    }

    #[test]
    fn test_mempool_transactions_priority() {
        let db_dir = tempfile::tempdir().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path, 10, 3600, 600);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        create_account(sender, 5000, db_path);
        // Add transactions with different fees
        let transactions = vec![
            Transaction {
                sender: sender.clone(),
                recipient: recipient.clone(),
                amount: 10,
                gas: 100,
                fee: 50,  // Low fee
                signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
                verifying_key: signing_key.verifying_key(),
                timestamp: 0,
            },
            Transaction {
                sender: sender.clone(),
                recipient: recipient.clone(),
                amount: 20,
                gas: 200,
                fee: 100,  // Medium fee
                signature: sign(sender.clone(), recipient.clone(), 20, &signing_key),
                verifying_key: signing_key.verifying_key(),
                timestamp: 0,
            },
            Transaction {
                sender: sender.clone(),
                recipient: recipient.clone(),
                amount: 30,
                gas: 300,
                fee: 200,  // High fee
                signature: sign(sender.clone(), recipient.clone(), 30, &signing_key),
                verifying_key: signing_key.verifying_key(),
                timestamp: 0,
            },
        ];

        // Add transactions to the mempool
        for transaction in transactions {
            mempool.add_transaction(transaction).unwrap();
        }

        // Retrieve the transactions from the mempool
        let mempool_transactions = mempool.get_transactions_priority();

        // Verify the order of transactions in the transactions_priority vector
        assert_eq!(mempool_transactions.len(), 3);
        assert_eq!(mempool_transactions[0].fee, 200);  // Highest fee
        assert_eq!(mempool_transactions[1].fee, 100);  // Medium fee
        assert_eq!(mempool_transactions[2].fee, 50);   // Lowest fee
    }
}

