use near_sdk::{ext_contract,AccountId};

pub const TGAS: u64 = 1_000_000_000_000;
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

// Interface of this contract, for callbacks
#[ext_contract(this_contract)]
trait Callbacks {
  fn ft_transfer_callback(&mut self) -> bool;
}

// Validator interface, for cross-contract calls
#[ext_contract(ft_contracts)]
trait FtContract {
  #[payable]
  fn ft_transfer(
    &mut self, 
    receiver_id: AccountId, 
    amount: u128, 
    memo: Option<String>
);
}