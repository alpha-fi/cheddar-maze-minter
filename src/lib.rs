use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near, AccountId, PanicOnDefault};

mod storage;

use storage::*;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    cheddar: AccountId,
    total_limit: u128,
    user_limit: u128,
    user_mints: UnorderedMap<AccountId, UserDailyMint>,
}

#[near]
impl Contract {
    #[init]
    pub fn new(cheddar: AccountId, total_limit: U128, user_limit: U128) -> Self {
        Self {
            cheddar,
            total_limit: total_limit.into(),
            user_limit: user_limit.into(),
            user_mints: UnorderedMap::new(StorageKey::UserMints),
        }
    }

    //
    // QUERIES
    //

    pub fn config(&self) -> String {
        self.total_limit.to_string()
    }

    //
    // TRANSACTIONS
    //

    pub fn mint(&mut self, amount: U128) {
        log!("minting: {:?}", amount);
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn set_then_get_greeting() {
    //     let mut contract = Contract::default();
    //     contract.set_greeting("howdy".to_string());
    //     assert_eq!(contract.get_greeting(), "howdy");
    // }
}
