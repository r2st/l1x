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
        let mempool = Mempool::new(100, 1000, db_path);
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
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
        };
        assert!(mempool.add_transaction(transaction).is_ok());
    }

    #[test]
    fn test_mempool_add_transaction_invalid_sender() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path);
        let sender: Address = [1u8; 32].into();
        let recipient: Address = [2u8; 32].into();
        let sec_bytes = &[1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(sec_bytes);
        let transaction = Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 10,
            gas: 100,
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
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
        let mempool = Mempool::new(1, 1000, db_path);
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
            signature: sign(sender1.clone(), recipient1.clone(), 10, &signing_key1),
            verifying_key: signing_key1.verifying_key(),
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
            signature: sign(sender2.clone(), recipient2.clone(), 10, &signing_key2),
            verifying_key: signing_key2.verifying_key(),
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
        let mempool = Mempool::new(100, 1000, db_path);
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
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
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
        let mempool = Mempool::new(100, 1000, db_path);
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
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key1),
            verifying_key: signing_key.verifying_key(),
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
        let mempool = Mempool::new(100, 1000, db_path);
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
            signature: sign(sender.clone(), recipient.clone(), 10000, &signing_key),
            verifying_key: signing_key.verifying_key(),
        };
        assert_eq!(
            mempool.add_transaction(transaction).err().unwrap().to_string(),
            "Insufficient balance to cover the transaction"
        );
    }

    #[test]
    fn test_mempool_remove_transaction() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();
        let mempool = Mempool::new(100, 1000, db_path);
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
            signature: sign(sender.clone(), recipient.clone(), 10, &signing_key),
            verifying_key: signing_key.verifying_key(),
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
        let mempool = Mempool::new(100, 1000, db_path);
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
            signature: sign(sender1.clone(), recipient1.clone(), 10, &signing_key1),
            verifying_key: signing_key1.verifying_key(),
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
            signature: sign(sender2.clone(), recipient2.clone(), 5, &signing_key2),
            verifying_key: signing_key2.verifying_key(),
        };
        assert!(mempool.add_transaction(transaction1.clone()).is_ok());
        assert!(mempool.add_transaction(transaction2.clone()).is_ok());
        let transactions = mempool.get_transactions();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.contains(&transaction1));
        assert!(transactions.contains(&transaction2));
    }
}
