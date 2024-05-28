use near_sdk::{near, BorshStorageKey};

#[derive(BorshStorageKey)]
#[near]
pub enum StorageKey {
    UserMints,
}

#[near(serializers=[borsh, json])]
pub struct UserDailyMint {
    day: u64,
    minted: u128,
}
