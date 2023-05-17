use primitives::{Address, Balance};
use crate::account_state::AccountState;
use anyhow::{Error, anyhow};

pub struct Account {
    pub address: Address,
    pub balance: Balance,
}

impl Account {
    #[allow(dead_code)]
    pub fn new(address: Address) -> Self {
        Account {
            address,
            balance: 0,
        }
    }
    #[allow(dead_code)]
    pub fn get_balance(&self) -> Balance {
        self.balance
    }
    #[allow(dead_code)]
    pub fn transfer(
        &mut self,
        to: Address,
        amount: Balance,
        account_state: &AccountState,
    ) -> Result<(), Error> {
        if !self.has_sufficient_balance(amount) {
            return Err(anyhow!("Insufficient balance"));
        }

        self.balance -= amount;
        account_state.update_account(&self)?;

        let mut to_account = account_state.get_account(to)?;
        to_account.balance += amount;
        account_state.update_account(&to_account)?;

        Ok(())
    }
    #[allow(dead_code)]
    pub fn has_sufficient_balance(&self, amount: Balance) -> bool {
        self.balance >= amount
    }
}

