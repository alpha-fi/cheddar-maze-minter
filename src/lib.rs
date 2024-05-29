use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, log, near, require, AccountId, Gas, NearToken, PanicOnDefault};

mod storage;

use storage::*;

const DAY_MS: u64 = 24 * 3600 * 1000;

#[ext_contract(ext_cheddar)]
pub trait ExtSelf {
    fn ft_mint(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

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
    #[payable]
    pub fn mint(&mut self, recipient: AccountId, amount: U128) {
        self.assert_minter();
        require!(
            env::prepaid_gas() >= Gas::from_tgas(30),
            "at least 30tgas must be attached"
        );
        let day = env::block_timestamp_ms() / DAY_MS;
        let amount: u128 = amount.into();
        if day == self.last_mint_day {
            self.daily_mints += amount;
            require!(
                self.daily_mints <= self.daily_quota,
                format!(
                    "total daily mint quota exceeded. Used: {}",
                    self.daily_mints
                )
            );
            require!(amount < self.user_quota, "amount above user quota");
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

        ext_cheddar::ext(self.cheddar.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_mint(recipient, amount.into(), None);
        // ext_cheddar::ft_mint(
        //     self.cheddar.clone(),
        //     1.into(),
        //     None,
        //     &self.cheddar_id,
        //     ONE_YOCTO,
        //     GAS_FOR_FT_TRANSFER,
        // );
    }

    pub fn admin_toggle_active(&mut self) {
        require!(
            env::predecessor_account_id() == self.admin,
            "must be an admin"
        );
        self.active = !self.active;
        log!("setting contract active={}", self.active);
    }

    pub fn admin_change_minter(&mut self, minter: AccountId) {
        require!(
            env::predecessor_account_id() == self.admin,
            "must be an admin"
        );
        log!("setting new minter: {}", minter);
        self.minter = minter;
    }

    //
    // INTERNAL
    //

    pub fn assert_minter(&self) {
        require!(
            env::predecessor_account_id() == self.minter,
            "must be a minter"
        );
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
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    /// 1ms in nano seconds
    const MSECOND: u64 = 1_000_000;

    fn cheddar() -> AccountId {
        // AccountId::new_unvalidated(a.to_owned())
        "cheddar.near".parse().unwrap()
    }
    fn admin() -> AccountId {
        "admin.near".parse().unwrap()
    }
    fn minter() -> AccountId {
        "minter.near".parse().unwrap()
    }
    fn alice() -> AccountId {
        "alice.near".parse().unwrap()
    }
    fn bob() -> AccountId {
        "bob.near".parse().unwrap()
    }
    fn charlie() -> AccountId {
        "charlie.near".parse().unwrap()
    }

    fn setup() -> (VMContext, Contract) {
        let ctx = VMContextBuilder::new()
            .predecessor_account_id(minter())
            .block_timestamp(DAY_MS * MSECOND)
            .build();
        testing_env!(ctx.clone());
        let ctr = Contract::new(cheddar(), admin(), minter(), U128::from(22), U128::from(10));
        (ctx, ctr)
    }

    #[test]
    fn toggle_active() {
        let (mut ctx, mut ctr) = setup();
        ctx.predecessor_account_id = admin();
        testing_env!(ctx);

        assert_eq!(ctr.active, true, "must be active by default");
        ctr.admin_toggle_active();
        assert_eq!(ctr.active, false);
        ctr.admin_toggle_active();
        assert_eq!(ctr.active, true);
    }

    #[test]
    #[should_panic(expected = "must be an admin")]
    fn toggle_active_not_admin() {
        let (_, mut ctr) = setup();
        ctr.admin_toggle_active();
    }

    #[test]
    #[should_panic(expected = "must be a minter")]
    fn mint_not_minter() {
        let (mut ctx, mut ctr) = setup();
        ctx.predecessor_account_id = admin();
        testing_env!(ctx);
        ctr.mint(alice(), 1.into())
    }

    #[test]
    fn mint() {
        let (mut ctx, mut ctr) = setup();
        ctr.mint(alice(), 1.into());
        ctr.mint(alice(), 9.into());
        assert_eq!(
            ctr.user_mints.get(&alice()).unwrap(),
            UserDailyMint { minted: 10, day: 1 }
        );
        assert_eq!(ctr.daily_mints, 10);
        assert_eq!(ctr.last_mint_day, 1);

        ctr.mint(bob(), 4.into());
        // recheck alice
        assert_eq!(
            ctr.user_mints.get(&alice()).unwrap(),
            UserDailyMint { minted: 10, day: 1 }
        );
        assert_eq!(
            ctr.user_mints.get(&bob()).unwrap(),
            UserDailyMint { minted: 4, day: 1 }
        );

        ctr.mint(charlie(), 8.into());
        assert_eq!(ctr.last_mint_day, 1);
        assert_eq!(ctr.daily_mints, 22);

        ctx.block_timestamp += DAY_MS * MSECOND;
        testing_env!(ctx.clone());
        ctr.mint(alice(), 7.into());
        assert_eq!(ctr.daily_mints, 7);
        assert_eq!(ctr.last_mint_day, 2);

        // same day but a bit later
        ctx.block_timestamp += DAY_MS / 2 * MSECOND;
        testing_env!(ctx.clone());
        ctr.mint(alice(), 2.into());
        assert_eq!(ctr.daily_mints, 9);
        assert_eq!(ctr.last_mint_day, 2);
        assert_eq!(
            ctr.user_mints.get(&alice()).unwrap(),
            UserDailyMint { minted: 9, day: 2 }
        );

        // few days later
        ctx.block_timestamp += 3 * DAY_MS * MSECOND;
        testing_env!(ctx.clone());
        ctr.mint(alice(), 2.into());
        assert_eq!(ctr.daily_mints, 2);
        assert_eq!(ctr.last_mint_day, 5);
        assert_eq!(
            ctr.user_mints.get(&alice()).unwrap(),
            UserDailyMint { minted: 2, day: 5 }
        );

        assert_eq!(
            ctr.user_mints.get(&bob()).unwrap(),
            UserDailyMint { minted: 4, day: 1 },
            "bob should be still in the old day"
        );
        ctr.mint(bob(), 1.into());
        assert_eq!(
            ctr.user_mints.get(&bob()).unwrap(),
            UserDailyMint { minted: 1, day: 5 },
        );
        assert_eq!(ctr.daily_mints, 3);
        assert_eq!(ctr.last_mint_day, 5);
    }

    #[test]
    #[should_panic(expected = "user daily mint quota used")]
    fn mint_exceed_user_quota() {
        let (_, mut ctr) = setup();
        ctr.mint(alice(), 5.into());
        ctr.mint(alice(), 6.into());
    }

    #[test]
    #[should_panic(expected = "total daily mint quota exceeded. Used: 24")]
    fn mint_exceed_total_quota() {
        let (_, mut ctr) = setup();
        ctr.mint(alice(), 8.into());
        ctr.mint(bob(), 8.into());
        ctr.mint(charlie(), 8.into());
    }
}
