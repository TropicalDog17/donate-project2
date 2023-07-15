use crate::Contract;
use crate::ContractExt;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Promise};

pub const STORAGE_COST: u128 = 1_000_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    pub account_id: AccountId,
    pub total_amount: U128,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn donate(&mut self) -> U128 {
        // Get who is calling the method and how much $NEAR they attached
        let donor: AccountId = env::predecessor_account_id();
        let donation_amount: Balance = env::attached_deposit();

        let mut donated_so_far = self.donations.get(&donor).unwrap_or(0);

        let to_transfer: Balance = if donated_so_far == 0 {
            assert!(donation_amount > STORAGE_COST, "Attach at least {} yoctoNEAR", STORAGE_COST);

            donation_amount - STORAGE_COST
        } else {
            donation_amount
        };

        donated_so_far += donation_amount;
        self.donations.insert(&donor, &donated_so_far);

        log!(
            "Thank you {} for donating {}! You donated a total of {}",
            donor.clone(),
            donation_amount,
            donated_so_far
        );

        // Send the money to the beneficiary
        Promise::new(self.beneficiary.clone()).transfer(to_transfer);

        // Return the total amount donated so far
        U128(donated_so_far)
    }

    // Public - get donation by account ID
    pub fn get_donation_for_account(&self, account_id: AccountId) -> Donation {
        Donation {
            account_id: account_id.clone(),
            total_amount: U128(self.donations.get(&account_id).unwrap_or(0)),
        }
    }

    // Public - get total number of donors
    pub fn number_of_donors(&self) -> u64 {
        self.donations.len()
    }

    pub fn get_donations(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Donation> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.donations
            .keys()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|account| self.get_donation_for_account(account))
            .collect()
    }
}
