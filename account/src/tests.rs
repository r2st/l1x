#[cfg(test)]
mod tests {
    use crate::account::*;
    use crate::account_state::*;
    use primitives::*;
    use tempfile::TempDir;

    #[test]
    fn test_account_new() {
        let address: Address = [0; 32];
        let account = Account::new(address);

        assert_eq!(account.address, address);
        assert_eq!(account.balance, 0);
    }

    #[test]
    fn test_account_get_balance() {
        let address: Address = [0; 32];
        let account = Account::new(address);

        assert_eq!(account.get_balance(), 0);
    }

    #[test]
    fn test_account_transfer_successful() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();

        let account1_address: Address = [0; 32];
        let account2_address: Address = [1; 32];

        let account_state = AccountState::new(db_path).unwrap();
        let mut account1 = account_state
            .get_account(account1_address)
            .unwrap();

        account1.balance = 100;

        assert!(account1.transfer(account2_address, 50, &account_state).is_ok());
        assert_eq!(account1.get_balance(), 50);

        let account2 = account_state
            .get_account(account2_address)
            .unwrap();
        assert_eq!(account2.get_balance(), 50);
    }

    #[test]
    fn test_account_transfer_insufficient_balance() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();

        let account1_address: Address = [0; 32];
        let account2_address: Address = [1; 32];

        let account_state = AccountState::new(db_path).unwrap();
        let mut account1 = account_state
            .get_account(account1_address)
            .unwrap();

        account1.balance = 30;

        assert!(account1.transfer(account2_address, 50, &account_state).is_err());
        assert_eq!(account1.get_balance(), 30);
    }

    #[test]
    fn test_account_state_get_account_existing() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();

        let account_address: Address = [0; 32];
        let account_balance: Balance = 100;

        let account_state = AccountState::new(db_path).unwrap();
        let mut account = Account::new(account_address);
        account.balance = account_balance;
        account_state.update_account(&account).unwrap();

        let retrieved_account = account_state
            .get_account(account_address)
            .unwrap();

        assert_eq!(retrieved_account.address, account_address);
        assert_eq!(retrieved_account.balance, account_balance);
    }

    #[test]
    fn test_account_state_get_account_nonexistent() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str().unwrap();

        let account_address: Address = [0; 32];

        let account_state = AccountState::new(db_path).unwrap();

        let retrieved_account = account_state
            .get_account(account_address)
            .unwrap();

        assert_eq!(retrieved_account.address, account_address);
        assert_eq!(retrieved_account.balance, 0);
    }

    #[test]
    fn test_account_state_update_account() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str();
        let account_address: Address = [0; 32];
        let account_balance: Balance = 100;

        let account_state = AccountState::new(db_path.unwrap()).unwrap();
        let mut updated_account = Account::new(account_address);
        updated_account.balance = account_balance;

        account_state.update_account(&updated_account).unwrap();

        let retrieved_account = account_state
            .get_account(account_address)
            .unwrap();

        assert_eq!(retrieved_account.address, account_address);
        assert_eq!(retrieved_account.balance, account_balance);
    }

    #[test]
    fn test_is_sufficient_balance_enough_balance() {
        let mut account = Account::new([0; 32]);
        account.balance = 100;

        assert!(account.has_sufficient_balance(50));
        assert!(account.has_sufficient_balance(100));
    }

    #[test]
    fn test_is_sufficient_balance_insufficient_balance() {
        let mut account = Account::new([0; 32]);
        account.balance = 50;

        assert!(!account.has_sufficient_balance(100));
        assert!(account.has_sufficient_balance(50));
        assert!(account.has_sufficient_balance(0));
    }

    #[test]
    fn test_is_valid_account_existing_account() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str();

        let account_state = AccountState::new(db_path.unwrap()).unwrap();

        let address: Address = [1u8; 32].into();

        // Create an account and update it in the database
        let account = Account {
            address: address.clone(),
            balance: 100,
        };
        account_state.update_account(&account).unwrap();

        // Check if the account is considered valid
        assert!(account_state.is_valid_account(address));
    }

    #[test]
    fn test_is_valid_account_nonexistent_account() {
        let db_dir = TempDir::new().unwrap();
        let db_path = db_dir.path().to_str();

        let account_state = AccountState::new(db_path.unwrap()).unwrap();

        let address: Address = [1u8; 32].into();

        // Check if the account is considered invalid
        assert!(!account_state.is_valid_account(address));
    }
}
