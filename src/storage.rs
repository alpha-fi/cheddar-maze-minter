use near_sdk::{json_types::U128, near, AccountId, BorshStorageKey};

#[derive(BorshStorageKey)]
#[near]
pub enum StorageKey {
    UserMints,
}

#[near(serializers=[borsh, json])]
pub struct UserDailyMint {
    pub day: u64,
    pub minted: u128,
}

#[near(serializers=[json])]
pub struct Config {
    pub minter: AccountId,
    pub active: bool,
    pub daily_quota: U128,
    pub user_quota: U128,
}
