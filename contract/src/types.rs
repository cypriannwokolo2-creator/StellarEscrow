use soroban_sdk::{contracttype, Address, String};

pub const MAX_METADATA_SIZE: u32 = 1024;
pub const MAX_INSURANCE_PREMIUM_BPS: u32 = 1000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionalMetadata {
    None,
    Some(String),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
    AwaitingBridge,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
    Partial(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArbitrationConfig {
    Single(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub arbitrator: Option<Address>,
    pub status: TradeStatus,
    pub expiry_time: Option<u64>,
    pub currency: Address,
    pub metadata: OptionalMetadata,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KycStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserCompliance {
    pub kyc_status: KycStatus,
    pub aml_cleared: bool,
    pub jurisdiction: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainInfo {
    pub source_chain: String,
    pub source_tx_hash: String,
    pub expires_at_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePolicy {
    pub provider: Address,
    pub premium: u64,
    pub coverage: u64,
    pub claimed: bool,
}
