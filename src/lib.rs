use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near, require, AccountId, PanicOnDefault};

mod storage;

use storage::*;

const DAY_MS: u64 = 24 * 3600 * 1000;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    cheddar: AccountId,
    admin: AccountId,
    minter: AccountId,
    active: bool,
    daily_quota: u128,
    user_quota: u128,
    user_mints: UnorderedMap<AccountId, UserDailyMint>,
    daily_mints: u128,
    last_mint_day: u64,
}

#[near]
impl Contract {
    #[init]
    pub fn new(
        cheddar: AccountId,
        admin: AccountId,
        minter: AccountId,
        total_limit: U128,
        user_limit: U128,
    ) -> Self {
        Self {
            cheddar,
            admin,
            minter,
            active: true,
            daily_quota: total_limit.into(),
            user_quota: user_limit.into(),
            user_mints: UnorderedMap::new(StorageKey::UserMints),
            daily_mints: 0,
            last_mint_day: 0,
        }
    }

    //
    // QUERIES
    //

    pub fn config(&self) -> Config {
        Config {
            minter: self.minter.clone(),
            active: self.active,
            daily_quota: self.daily_quota.into(),
            user_quota: self.user_quota.into(),
        }
    }

    //
    // TRANSACTIONS
    //

    /// only minter can mint
    pub fn mint(&mut self, recipient: AccountId, amount: U128) {
        self.assert_minter();
        let day = env::block_timestamp_ms() % DAY_MS;
        let amount: u128 = amount.into();
        if day == self.last_mint_day {
            self.daily_mints += amount;
            require!(
                self.daily_mints <= self.daily_quota && amount <= self.user_quota,
                "total daily mint quota used"
            );
        } else {
            self.last_mint_day = day;
            self.daily_mints = amount;
        }

        let mut minted = 0;
        match self.user_mints.get(&recipient) {
            None => (),
            Some(x) => {
                if x.day == day {
                    minted = x.minted;
                }
            }
        }
        minted += amount;
        require!(minted <= self.user_quota, "user daily mint quota used");
        self.user_mints
            .insert(&recipient, &UserDailyMint { day, minted });
        // TODO: cross contract call
    }

    pub fn admin_toggle_active(&mut self) {
        require!(env::current_account_id() == self.admin, "must be an admin");
        self.active = !self.active;
        log!("setting contract active={}", self.active);
    }

    //
    // INTERNAL
    //

    pub fn assert_minter(&self) {
        require!(env::current_account_id() == self.minter, "must be a minter");
        require!(self.active, "contract is disactivated");
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
