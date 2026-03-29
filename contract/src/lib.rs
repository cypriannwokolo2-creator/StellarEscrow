#![no_std]

#[cfg(test)]
extern crate std;

mod analytics;
mod errors;
mod events;
mod storage;
pub mod types;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};

pub use analytics::{
    AnalyticsResult, ArbitratorMetrics, PeriodAnalytics, PlatformMetrics, PlatformStats,
    SuccessRateStats, TimeWindow, VolumeStats,
};
pub use errors::ContractError;
pub use types::{
    ArbitrationConfig, CrossChainInfo, DisputeResolution, InsurancePolicy, KycStatus,
    OptionalMetadata, Trade, TradeStatus, UserCompliance, MAX_INSURANCE_PREMIUM_BPS,
    MAX_METADATA_SIZE,
};

fn require_initialized(env: &Env) -> Result<(), ContractError> {
    if !storage::is_initialized(env) {
        return Err(ContractError::NotInitialized);
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if storage::is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

fn validate_metadata(metadata: &OptionalMetadata) -> Result<(), ContractError> {
    match metadata {
        OptionalMetadata::None => Ok(()),
        OptionalMetadata::Some(value) => {
            let len = value.len();
            if len == 0 {
                Err(ContractError::InvalidMetadata)
            } else if len > MAX_METADATA_SIZE {
                Err(ContractError::MetadataValueTooLong)
            } else {
                Ok(())
            }
        }
    }
}

fn validate_user_compliance(env: &Env, user: &Address, amount: u64) -> Result<(), ContractError> {
    let compliance = storage::get_user_compliance(env, user).ok_or(ContractError::ComplianceDataMissing)?;
    if compliance.kyc_status != KycStatus::Verified {
        return Err(ContractError::KycNotVerified);
    }
    if !compliance.aml_cleared {
        return Err(ContractError::AmlNotCleared);
    }
    if !storage::is_jurisdiction_allowed(env, &compliance.jurisdiction) {
        return Err(ContractError::JurisdictionRestricted);
    }
    let user_limit = storage::get_user_trade_limit(env, user);
    if user_limit > 0 && amount > user_limit {
        return Err(ContractError::TradeAmountLimitExceeded);
    }
    if amount > storage::get_global_trade_limit(env) {
        return Err(ContractError::TradeAmountLimitExceeded);
    }
    Ok(())
}

fn calc_fee(env: &Env, amount: u64) -> Result<u64, ContractError> {
    amount
        .checked_mul(storage::get_fee_bps(env)? as u64)
        .ok_or(ContractError::Overflow)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)
}

fn usdc_client<'a>(env: &'a Env) -> Result<token::Client<'a>, ContractError> {
    Ok(token::Client::new(env, &storage::get_usdc_token(env)?))
}

#[contract]
pub struct StellarEscrowContract;

#[contractimpl]
impl StellarEscrowContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_token: Address,
        fee_bps: u32,
    ) -> Result<(), ContractError> {
        if storage::is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        if fee_bps > 10_000 {
            return Err(ContractError::InvalidFeeBps);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_usdc_token(&env, &usdc_token);
        storage::set_fee_bps(&env, fee_bps);
        storage::set_trade_counter(&env, 0);
        storage::set_accumulated_fees(&env, 0);
        storage::set_version(&env, 1);
        storage::set_initialized(&env);
        Ok(())
    }

    pub fn register_arbitrator(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::save_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_registered(&env, arbitrator);
        Ok(())
    }

    pub fn remove_arbitrator_fn(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::remove_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_removed(&env, arbitrator);
        Ok(())
    }

    pub fn is_arbitrator_registered(env: Env, arbitrator: Address) -> bool {
        storage::has_arbitrator(&env, &arbitrator)
    }

    pub fn update_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if fee_bps > 10_000 {
            return Err(ContractError::InvalidFeeBps);
        }
        storage::get_admin(&env)?.require_auth();
        storage::set_fee_bps(&env, fee_bps);
        events::emit_fee_updated(&env, fee_bps);
        Ok(())
    }

    pub fn get_platform_fee_bps(env: Env) -> Result<u32, ContractError> {
        storage::get_fee_bps(&env)
    }

    pub fn set_user_compliance(
        env: Env,
        admin: Address,
        user: Address,
        compliance: UserCompliance,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::save_user_compliance(&env, &user, &compliance);
        events::emit_compliance_updated(&env, user);
        Ok(())
    }

    pub fn get_user_compliance(env: Env, user: Address) -> Option<UserCompliance> {
        storage::get_user_compliance(&env, &user)
    }

    pub fn set_user_trade_limit(
        env: Env,
        admin: Address,
        user: Address,
        limit: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::set_user_trade_limit(&env, &user, limit);
        Ok(())
    }

    pub fn set_jurisdiction_rule(
        env: Env,
        admin: Address,
        jurisdiction: String,
        allowed: bool,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::set_jurisdiction_rule(&env, &jurisdiction, allowed);
        Ok(())
    }

    pub fn set_global_trade_limit(
        env: Env,
        admin: Address,
        limit: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if admin != storage::get_admin(&env)? {
            return Err(ContractError::Unauthorized);
        }
        admin.require_auth();
        storage::set_global_trade_limit(&env, limit);
        Ok(())
    }

    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        metadata: OptionalMetadata,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        validate_metadata(&metadata)?;
        seller.require_auth();
        validate_user_compliance(&env, &seller, amount)?;
        validate_user_compliance(&env, &buyer, amount)?;
        let arbitration = match arbitrator {
            Some(addr) => {
                if !storage::has_arbitrator(&env, &addr) {
                    return Err(ContractError::ArbitratorNotRegistered);
                }
                Some(addr)
            }
            None => None,
        };
        let trade_id = storage::increment_trade_counter(&env)?;
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee: calc_fee(&env, amount)?,
            arbitrator: arbitration,
            status: TradeStatus::Created,
            expiry_time: None,
            currency: storage::get_usdc_token(&env)?,
            metadata,
        };
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller.clone(), buyer.clone(), amount, trade.currency.clone());
        events::emit_compliance_passed(&env, trade_id, seller, buyer, amount);
        analytics::on_trade_created(&env, amount, &trade.seller, &trade.buyer);
        Ok(trade_id)
    }

    pub fn fund_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        token::Client::new(&env, &trade.currency).transfer(
            &trade.buyer,
            &env.current_contract_address(),
            &(trade.amount as i128),
        );
        trade.status = TradeStatus::Funded;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_funded(&env, trade_id);
        analytics::on_trade_funded(&env);
        Ok(())
    }

    pub fn complete_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Completed;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_completed(&env, trade_id);
        Ok(())
    }

    pub fn confirm_receipt(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let payout = trade
            .amount
            .checked_sub(trade.fee)
            .ok_or(ContractError::Overflow)?;
        let token_client = token::Client::new(&env, &trade.currency);
        let contract_balance = token_client.balance(&env.current_contract_address());
        let required_balance = trade.amount as i128;
        if contract_balance < required_balance {
            token_client.transfer(
                &trade.buyer,
                &env.current_contract_address(),
                &(required_balance - contract_balance),
            );
        }
        token_client.transfer(&env.current_contract_address(), &trade.seller, &(payout as i128));
        storage::add_accumulated_fees(&env, trade.fee)?;
        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);
        analytics::on_trade_completed(&env, trade.fee);
        Ok(())
    }

    pub fn cancel_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_trade_cancelled(&env, trade_id);
        analytics::on_trade_cancelled(&env);
        Ok(())
    }

    pub fn raise_dispute(
        env: Env,
        trade_id: u64,
        caller: Address,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        if trade.arbitrator.is_none() {
            return Err(ContractError::ArbitratorNotRegistered);
        }
        caller.require_auth();
        if caller != trade.buyer && caller != trade.seller {
            return Err(ContractError::Unauthorized);
        }
        trade.status = TradeStatus::Disputed;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_dispute_raised(&env, trade_id, caller);
        analytics::on_trade_disputed(&env);
        Ok(())
    }

    pub fn resolve_dispute(
        env: Env,
        trade_id: u64,
        resolution: DisputeResolution,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed {
            return Err(ContractError::InvalidStatus);
        }
        let arbitrator = match trade.arbitrator.clone() {
            Some(addr) => addr,
            None => return Err(ContractError::NoArbitrator),
        };
        arbitrator.require_auth();
        let net = trade
            .amount
            .checked_sub(trade.fee)
            .ok_or(ContractError::Overflow)?;
        let token_client = token::Client::new(&env, &trade.currency);
        match resolution.clone() {
            DisputeResolution::ReleaseToBuyer => {
                token_client.transfer(&env.current_contract_address(), &trade.buyer, &(net as i128));
                events::emit_dispute_resolved(&env, trade_id, resolution, trade.buyer);
            }
            DisputeResolution::ReleaseToSeller => {
                token_client.transfer(&env.current_contract_address(), &trade.seller, &(net as i128));
                events::emit_dispute_resolved(&env, trade_id, resolution, trade.seller);
            }
            DisputeResolution::Partial(buyer_bps) => {
                if buyer_bps > 10_000 {
                    return Err(ContractError::InvalidSplitBps);
                }
                let buyer_amount = net
                    .checked_mul(buyer_bps as u64)
                    .ok_or(ContractError::Overflow)?
                    .checked_div(10_000)
                    .ok_or(ContractError::Overflow)?;
                let seller_amount = net
                    .checked_sub(buyer_amount)
                    .ok_or(ContractError::Overflow)?;
                if buyer_amount > 0 {
                    token_client.transfer(
                        &env.current_contract_address(),
                        &trade.buyer,
                        &(buyer_amount as i128),
                    );
                }
                if seller_amount > 0 {
                    token_client.transfer(
                        &env.current_contract_address(),
                        &trade.seller,
                        &(seller_amount as i128),
                    );
                }
                events::emit_partial_resolved(&env, trade_id, buyer_amount, seller_amount, trade.fee);
            }
        }
        storage::add_accumulated_fees(&env, trade.fee)?;
        Ok(())
    }

    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, ContractError> {
        storage::get_trade(&env, trade_id)
    }

    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = storage::get_admin(&env)?;
        admin.require_auth();
        let fees = storage::get_accumulated_fees(&env)?;
        if fees == 0 {
            return Err(ContractError::NoFeesToWithdraw);
        }
        usdc_client(&env)?.transfer(&env.current_contract_address(), &to, &(fees as i128));
        storage::set_accumulated_fees(&env, 0);
        events::emit_fees_withdrawn(&env, fees, to);
        Ok(())
    }

    pub fn get_accumulated_fees(env: Env) -> Result<u64, ContractError> {
        storage::get_accumulated_fees(&env)
    }

    pub fn pause(env: Env) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = storage::get_admin(&env)?;
        admin.require_auth();
        storage::set_paused(&env, true);
        events::emit_paused(&env, admin);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let admin = storage::get_admin(&env)?;
        admin.require_auth();
        storage::set_paused(&env, false);
        events::emit_unpaused(&env, admin);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    pub fn version(env: Env) -> u32 {
        storage::get_version(&env)
    }

    pub fn migrate(env: Env, expected_version: u32) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let current = storage::get_version(&env);
        if current != expected_version {
            return Err(ContractError::MigrationVersionMismatch);
        }
        let next = current.checked_add(1).ok_or(ContractError::Overflow)?;
        storage::set_version(&env, next);
        events::emit_migrated(&env, current, next);
        Ok(())
    }

    pub fn set_bridge_oracle(env: Env, oracle: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::set_bridge_oracle(&env, &oracle);
        events::emit_bridge_oracle_set(&env, oracle);
        Ok(())
    }

    pub fn create_cross_chain_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        source_chain: String,
        expiry_ledgers: u32,
    ) -> Result<u64, ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        if storage::get_bridge_oracle(&env).is_none() {
            return Err(ContractError::BridgeOracleNotSet);
        }
        seller.require_auth();
        let arbitration = match arbitrator {
            Some(addr) => {
                if !storage::has_arbitrator(&env, &addr) {
                    return Err(ContractError::ArbitratorNotRegistered);
                }
                Some(addr)
            }
            None => None,
        };
        let trade_id = storage::increment_trade_counter(&env)?;
        let expires_at_ledger = env
            .ledger()
            .sequence()
            .checked_add(expiry_ledgers)
            .ok_or(ContractError::Overflow)?;
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee: calc_fee(&env, amount)?,
            arbitrator: arbitration,
            status: TradeStatus::AwaitingBridge,
            expiry_time: None,
            currency: storage::get_usdc_token(&env)?,
            metadata: OptionalMetadata::None,
        };
        storage::save_trade(&env, trade_id, &trade);
        storage::save_cross_chain_info(
            &env,
            trade_id,
            &CrossChainInfo {
                source_chain: source_chain.clone(),
                source_tx_hash: String::from_str(&env, ""),
                expires_at_ledger,
            },
        );
        events::emit_bridge_trade_created(&env, trade_id, source_chain);
        analytics::on_trade_created(&env, amount, &trade.seller, &trade.buyer);
        Ok(trade_id)
    }

    pub fn confirm_bridge_deposit(
        env: Env,
        trade_id: u64,
        source_tx_hash: String,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let oracle = storage::get_bridge_oracle(&env).ok_or(ContractError::BridgeOracleNotSet)?;
        oracle.require_auth();
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::AwaitingBridge {
            return Err(ContractError::InvalidStatus);
        }
        let mut info = storage::get_cross_chain_info(&env, trade_id).ok_or(ContractError::TradeNotFound)?;
        if env.ledger().sequence() > info.expires_at_ledger {
            return Err(ContractError::BridgeTradeExpired);
        }
        info.source_tx_hash = source_tx_hash;
        storage::save_cross_chain_info(&env, trade_id, &info);
        trade.status = TradeStatus::Funded;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_bridge_deposit_confirmed(&env, trade_id);
        analytics::on_trade_funded(&env);
        Ok(())
    }

    pub fn expire_bridge_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let mut trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::AwaitingBridge {
            return Err(ContractError::InvalidStatus);
        }
        let info = storage::get_cross_chain_info(&env, trade_id).ok_or(ContractError::TradeNotFound)?;
        if env.ledger().sequence() <= info.expires_at_ledger {
            return Err(ContractError::BridgeTradeNotExpired);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        storage::save_trade(&env, trade_id, &trade);
        events::emit_bridge_trade_expired(&env, trade_id);
        analytics::on_trade_cancelled(&env);
        Ok(())
    }

    pub fn get_cross_chain_info(env: Env, trade_id: u64) -> Option<CrossChainInfo> {
        storage::get_cross_chain_info(&env, trade_id)
    }

    pub fn register_insurance_provider(
        env: Env,
        provider: Address,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::save_insurance_provider(&env, &provider);
        events::emit_insurance_provider_registered(&env, provider);
        Ok(())
    }

    pub fn remove_insurance_provider(env: Env, provider: Address) -> Result<(), ContractError> {
        require_initialized(&env)?;
        storage::get_admin(&env)?.require_auth();
        storage::remove_insurance_provider(&env, &provider);
        events::emit_insurance_provider_removed(&env, provider);
        Ok(())
    }

    pub fn is_insurance_provider_registered(env: Env, provider: Address) -> bool {
        storage::has_insurance_provider(&env, &provider)
    }

    pub fn purchase_insurance(
        env: Env,
        trade_id: u64,
        provider: Address,
        premium_bps: u32,
        coverage: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        require_not_paused(&env)?;
        if premium_bps > MAX_INSURANCE_PREMIUM_BPS {
            return Err(ContractError::InsurancePremiumTooHigh);
        }
        if !storage::has_insurance_provider(&env, &provider) {
            return Err(ContractError::InsuranceProviderNotRegistered);
        }
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let premium = trade
            .amount
            .checked_mul(premium_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10_000)
            .ok_or(ContractError::Overflow)?;
        usdc_client(&env)?.transfer(&trade.buyer, &provider, &(premium as i128));
        storage::save_insurance_policy(
            &env,
            trade_id,
            &InsurancePolicy {
                provider: provider.clone(),
                premium,
                coverage,
                claimed: false,
            },
        );
        events::emit_insurance_purchased(&env, trade_id, provider, premium, coverage);
        Ok(())
    }

    pub fn claim_insurance(
        env: Env,
        trade_id: u64,
        recipient: Address,
        payout: u64,
    ) -> Result<(), ContractError> {
        require_initialized(&env)?;
        let trade = storage::get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed && trade.status != TradeStatus::Completed {
            return Err(ContractError::InsuranceClaimNotEligible);
        }
        let mut policy = storage::get_insurance_policy(&env, trade_id).ok_or(ContractError::TradeNotInsured)?;
        if policy.claimed {
            return Err(ContractError::InsuranceAlreadyClaimed);
        }
        policy.provider.require_auth();
        let actual_payout = if payout > policy.coverage { policy.coverage } else { payout };
        usdc_client(&env)?.transfer(&policy.provider, &recipient, &(actual_payout as i128));
        policy.claimed = true;
        storage::save_insurance_policy(&env, trade_id, &policy);
        events::emit_insurance_claimed(&env, trade_id, actual_payout, recipient);
        Ok(())
    }

    pub fn get_insurance_policy(env: Env, trade_id: u64) -> Option<InsurancePolicy> {
        storage::get_insurance_policy(&env, trade_id)
    }

    pub fn get_platform_metrics(env: Env) -> PlatformMetrics {
        analytics::get_metrics(&env)
    }

    pub fn get_platform_stats(env: Env) -> PlatformStats {
        analytics::get_stats(&env)
    }

    pub fn get_arbitrator_analytics(env: Env, arbitrator: Address) -> ArbitratorMetrics {
        analytics::get_arb_metrics(&env, &arbitrator)
    }

    pub fn analytics_query(env: Env, window: TimeWindow) -> AnalyticsResult {
        analytics::analytics_query(&env, window)
    }

    pub fn get_volume_stats(env: Env, window: TimeWindow) -> VolumeStats {
        analytics::get_volume_stats(&env, window)
    }

    pub fn get_success_rate(env: Env) -> SuccessRateStats {
        analytics::get_success_rate(&env)
    }

    pub fn get_platform_usage(env: Env) -> analytics::PlatformUsage {
        analytics::get_platform_usage(&env)
    }

    pub fn get_analytics_by_period(env: Env, start_time: u64, end_time: u64) -> PeriodAnalytics {
        analytics::get_analytics_by_period(&env, start_time, end_time)
    }
}

#[cfg(test)]
mod test;
