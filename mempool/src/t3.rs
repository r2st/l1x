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

        // Cloning the necessary values from mempool for separate borrows
        let entries: HashMap<Address, HashMap<TimeStamp, Vec<Transaction>>> = mempool
            .clone()
            .into_iter()
            .map(|(address, transactions)| (address, transactions.clone()))
            .collect();

        let entry = mempool
            .entry(transaction.sender.clone())
            .or_insert_with(HashMap::new)
            .entry(current_time)
            .or_insert_with(Vec::new);

        let sender_transaction_count = entries
            .get(&transaction.sender)
            .map(|transactions| {
                transactions.values().flatten().filter(|timestamp| {
                    *timestamp >= &(current_time - self.time_frame_seconds)
                }).count()
            })
            .unwrap_or(0);

        if sender_transaction_count >= self.rate_limit {
            return Err(anyhow!("Rate limit exceeded for the sender"));
        }

        // Update the transaction timestamp for the sender
        entry.clear();
        entry.push(transaction);

        Ok(())
    }


    let entry = mempool
            .entry(transaction.sender.clone())
            .or_insert_with(HashMap::new)
            .entry(current_time)
            .or_insert_with(Vec::new);
        // Update the transaction timestamp for the sender
        entry.clear();
        entry.push(transaction);