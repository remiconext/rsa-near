use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen,AccountId,env,Balance,require};
use near_sdk::collections::{LookupMap,LookupSet};
use rsa::{RsaPublicKey,BigUint};
use rsa::PublicKey;
use rsa::padding::PaddingScheme;
use rsa::pkcs8::DecodePublicKey;
use rsa::pkcs1::DecodeRsaPublicKey;
use near_sdk::PromiseError;

pub mod external;
pub use crate::external::*;


#[derive(BorshSerialize, BorshDeserialize)]
struct Campain{
    owner: AccountId,
    total_supply: String,
    reward_contract: AccountId,
    already_win: LookupSet<Vec<u8>>
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    campains: LookupMap<u32, Campain>,
    owner_id: AccountId,
    pem: String
}

impl Default for Contract{
    fn default() -> Self{
        Self {
            campains: LookupMap::new(b"a"),
            owner_id: env::predecessor_account_id(),
            pem : "".to_string()
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {

    #[init]
    #[private]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            campains: LookupMap::new(b"a"),
            owner_id: env::predecessor_account_id(),
            pem : "".to_string()
        }
    }

    pub fn ft_on_transfer(&mut self,sender_id: String, amount: String, msg: String) -> String{
        let reward_contract = env::predecessor_account_id();
        self.register_new_campain( msg.parse::<u32>().unwrap(),amount, reward_contract, AccountId::try_from(sender_id).unwrap());
        String::from("0")
    }

    pub fn add_public_key(&mut self,pem: String){
        let sender_id = env::predecessor_account_id();
        assert!(self.owner_id == sender_id, "Only Owner can set public key");
        self.pem = pem;
    }

    pub fn change_owner(&mut self,new_owner:AccountId){
        let sender_id = env::predecessor_account_id();
        assert!(self.owner_id == sender_id, "Only Owner can set new owner");
        self.owner_id = new_owner;
    }

    pub fn get_rewards(&self,data: String, signature: Vec<u8>){
        let hash_signature = &self.verify_signature(data.clone(),signature.clone());
        
        let split = data.split("_");
        let vec_data: Vec<&str> = split.collect();

        let campain = self.campains.get(&vec_data[0].parse::<u32>().unwrap());
        if let None = campain{
            panic!("The campain doesn't exist");
        }
        let campain_unwrap = campain.unwrap();

        require!(!campain_unwrap.already_win.contains(&hash_signature),"Sorry you have already got your rewards");
        require!(vec_data[3].parse::<u64>().unwrap()<env::block_timestamp(),"Sorry you can't get your reward anymore");

        ft_contracts::ext(campain_unwrap.reward_contract.clone())
            .ft_transfer(env::predecessor_account_id(),vec_data[1].parse::<u128>().unwrap(),None)
        .then(
            Self::ext(env::current_account_id())
            .ft_transfer_callback(vec_data[0].parse::<u32>().unwrap(),signature)
        );
    }

    #[private]
    pub fn ft_transfer_callback(&mut self, campain: u32,signature: Vec<u8>,#[callback_result] call_result: Result<(), PromiseError>) -> bool {
        // Return whether or not the promise succeeded using the method outlined in external.rs
        if call_result.is_err() {
            self.campains.get(&campain).unwrap().already_win.insert_raw(&signature);
            return true;
        } else {
            return false;
        }
    }

    fn register_new_campain(&mut self, campain_id: u32, total_supply: String, reward_contract: AccountId,sender_id: AccountId){
        let new_campain = Campain{
            owner: sender_id,
            total_supply,
            reward_contract,
            already_win: LookupSet::new(b"a")
        };
        self.campains.insert(&campain_id,&new_campain);
    }

    fn verify_signature(&self,data: String, signature: Vec<u8>) -> Vec<u8>{
        let hash_signature = env::sha256(data.as_bytes());
        let public_key = RsaPublicKey::from_public_key_pem(&self.pem).unwrap();
        let padding = PaddingScheme::new_pkcs1v15_sign(None);
        require!(public_key.verify(padding,&hash_signature,&signature).is_ok(),"Sorry, invalid signature");
        hash_signature
    }

}


#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    const NEAR: u128 = 1000000000000000000000000;

    #[test]
    fn initializes() {
        let contract = Contract::new();
    }
    

    fn set_context(predecessor: &str, amount: Balance) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(amount);
    
        testing_env!(builder.build());
    }

}
