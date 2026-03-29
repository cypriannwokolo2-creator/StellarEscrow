use soroban_sdk::{Address, Env, String};

use crate::types::DisputeResolution;

pub fn emit_arbitrator_registered(_env: &Env, _arbitrator: Address) {}
pub fn emit_arbitrator_removed(_env: &Env, _arbitrator: Address) {}
pub fn emit_fee_updated(_env: &Env, _fee_bps: u32) {}
pub fn emit_compliance_failed(_env: &Env, _user: Address, _reason: &String) {}
pub fn emit_compliance_updated(_env: &Env, _user: Address) {}
pub fn emit_trade_created(
    _env: &Env,
    _trade_id: u64,
    _seller: Address,
    _buyer: Address,
    _amount: u64,
    _currency: Address,
) {
}
pub fn emit_compliance_passed(
    _env: &Env,
    _trade_id: u64,
    _seller: Address,
    _buyer: Address,
    _amount: u64,
) {
}
pub fn emit_trade_funded(_env: &Env, _trade_id: u64) {}
pub fn emit_trade_completed(_env: &Env, _trade_id: u64) {}
pub fn emit_trade_confirmed(_env: &Env, _trade_id: u64, _payout: u64, _fee: u64) {}
pub fn emit_dispute_raised(_env: &Env, _trade_id: u64, _raised_by: Address) {}
pub fn emit_dispute_resolved(
    _env: &Env,
    _trade_id: u64,
    _resolution: DisputeResolution,
    _recipient: Address,
) {
}
pub fn emit_partial_resolved(
    _env: &Env,
    _trade_id: u64,
    _buyer_amount: u64,
    _seller_amount: u64,
    _fee: u64,
) {
}
pub fn emit_trade_cancelled(_env: &Env, _trade_id: u64) {}
pub fn emit_fees_withdrawn(_env: &Env, _amount: u64, _to: Address) {}
pub fn emit_paused(_env: &Env, _admin: Address) {}
pub fn emit_unpaused(_env: &Env, _admin: Address) {}
pub fn emit_bridge_oracle_set(_env: &Env, _oracle: Address) {}
pub fn emit_bridge_trade_created(_env: &Env, _trade_id: u64, _source_chain: String) {}
pub fn emit_bridge_deposit_confirmed(_env: &Env, _trade_id: u64) {}
pub fn emit_bridge_trade_expired(_env: &Env, _trade_id: u64) {}
pub fn emit_insurance_provider_registered(_env: &Env, _provider: Address) {}
pub fn emit_insurance_provider_removed(_env: &Env, _provider: Address) {}
pub fn emit_insurance_purchased(
    _env: &Env,
    _trade_id: u64,
    _provider: Address,
    _premium: u64,
    _coverage: u64,
) {
}
pub fn emit_insurance_claimed(_env: &Env, _trade_id: u64, _payout: u64, _recipient: Address) {}
pub fn emit_migrated(_env: &Env, _from_version: u32, _to_version: u32) {}
