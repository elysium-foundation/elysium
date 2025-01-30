use crate::{Authorship, Balances};
use frame_support::traits::{Imbalance, OnUnbalanced};
use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::Currency;
use crate::AccountId;
use sp_std::{prelude::*};
type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
const PALLET_NAME: &'static str = "Elysium";
const STORAGE_NAME: &'static str = "FoundationWallet";
pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        if let Some(author) = Authorship::author() {
            Balances::resolve_creating(&author, amount);
        }
    }
}

pub struct Treasury;
impl OnUnbalanced<NegativeImbalance> for Treasury {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        let pallet_hash = sp_io::hashing::twox_128(PALLET_NAME.as_bytes());
        let storage_hash = sp_io::hashing::twox_128(STORAGE_NAME.as_bytes());
        type Data = AccountId;
        let mut final_key = Vec::new();
        final_key.extend_from_slice(&pallet_hash);
        final_key.extend_from_slice(&storage_hash);
        let acc=frame_support::storage::unhashed::get::<Data>(&final_key);
        match acc{
            None => (),
            Some(quotient) => {
                Balances::resolve_creating(&quotient, amount);
            },
        }
    }
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            let mut split = fees.ration(70, 30);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 70% to treasury, 30% to block author (though this can be anything)
                tips.ration_merge_into(70, 30, &mut split);
            }
            Treasury::on_unbalanced(split.0);
            Author::on_unbalanced(split.1);
        }
    }
}


pub struct DealWithEVMFees;
impl OnUnbalanced<NegativeImbalance> for DealWithEVMFees
{
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            let mut split = fees.ration(70, 30);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 70% to treasury, 30% to block author (though this can be anything)
                tips.ration_merge_into(70, 30, &mut split);
            }
            Treasury::on_unbalanced(split.0);
            Author::on_unbalanced(split.1);
        }
    }
    // this is called from pallet_evm for Ethereum-based transactions
    // (technically, it calls on_unbalanced, which calls this when non-zero)
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        let split = amount.ration(70, 30);
        Treasury::on_unbalanced(split.0);
        Author::on_unbalanced(split.1);
    }
}