use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
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
    pub created_at: u32,
    pub updated_at: u32,
    pub metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionRecord {
    pub trade_id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub status: TradeStatus,
    pub created_at: u32,
    pub updated_at: u32,
    pub metadata: Option<TradeMetadata>,
}

pub const METADATA_MAX_VALUE_LEN: u32 = 256;
pub const METADATA_MAX_ENTRIES: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeMetadata {
    pub entries: Vec<MetadataEntry>,
}

// ---------------------------------------------------------------------------
// Fee Tier System
// ---------------------------------------------------------------------------

pub const TIER_SILVER_THRESHOLD: u64 = 10_000_000_000;
pub const TIER_GOLD_THRESHOLD: u64 = 100_000_000_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserTierInfo {
    pub tier: UserTier,
    pub total_volume: u64,
    pub custom_fee_bps: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    pub bronze_fee_bps: u32,
    pub silver_fee_bps: u32,
    pub gold_fee_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryFilter {
    pub status: Option<TradeStatus>,
    pub from_ledger: Option<u32>,
    pub to_ledger: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

// ---------------------------------------------------------------------------
// Trade Templates
// ---------------------------------------------------------------------------

pub const TEMPLATE_NAME_MAX_LEN: u32 = 64;
pub const TEMPLATE_MAX_VERSIONS: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTerms {
    pub description: String,
    pub default_arbitrator: Option<Address>,
    pub fixed_amount: Option<u64>,
    pub default_metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateVersion {
    pub version: u32,
    pub terms: TemplateTerms,
    pub created_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub current_version: u32,
    pub versions: Vec<TemplateVersion>,
    pub active: bool,
    pub created_at: u32,
    pub updated_at: u32,
}

// ---------------------------------------------------------------------------
// User Management
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub username_hash: soroban_sdk::Bytes,
    pub contact_hash: soroban_sdk::Bytes,
    pub verification: VerificationStatus,
    pub registered_at: u32,
    pub updated_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPreference {
    pub key: soroban_sdk::String,
    pub value: soroban_sdk::String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserAnalytics {
    pub address: Address,
    pub total_trades: u32,
    pub trades_as_seller: u32,
    pub trades_as_buyer: u32,
    pub total_volume: u64,
    pub completed_trades: u32,
    pub disputed_trades: u32,
    pub cancelled_trades: u32,
}

// ---------------------------------------------------------------------------
// Admin Panel
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformAnalytics {
    pub total_trades: u64,
    pub total_volume: u64,
    pub total_fees_collected: u64,
    pub active_trades: u64,
    pub completed_trades: u64,
    pub disputed_trades: u64,
    pub cancelled_trades: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemConfig {
    pub fee_bps: u32,
    pub is_paused: bool,
    pub trade_counter: u64,
    pub accumulated_fees: u64,
}

// ---------------------------------------------------------------------------
// Trade Detail View
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineEntry {
    pub status: TradeStatus,
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeAction {
    Fund,
    Complete,
    ConfirmReceipt,
    RaiseDispute,
    Cancel,
    ResolveDispute,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeDetail {
    pub trade: Trade,
    pub timeline: Vec<TimelineEntry>,
    pub available_actions: Vec<TradeAction>,
    pub seller_payout: u64,
}

// ---------------------------------------------------------------------------
// Analytics Charts & Graphs
// ---------------------------------------------------------------------------

/// A single data point for a time-series chart (ledger bucket → value).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChartPoint {
    /// Ledger sequence representing the start of this bucket
    pub ledger: u32,
    /// Aggregated value for this bucket (volume, count, etc.)
    pub value: u64,
}

/// Trade volume chart data — bucketed by ledger range.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeChartData {
    /// Ordered data points (ascending ledger)
    pub points: Vec<ChartPoint>,
    /// Total volume across all points
    pub total_volume: u64,
    /// Total number of trades across all points
    pub total_trades: u64,
}

/// Success rate snapshot — completed vs total settled trades.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessRateData {
    pub completed: u64,
    pub disputed: u64,
    pub cancelled: u64,
    /// Basis points (0–10000): completed / (completed + disputed + cancelled) * 10000
    pub success_rate_bps: u32,
}

/// Per-status trade count breakdown for a status distribution chart.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusDistribution {
    pub created: u64,
    pub funded: u64,
    pub completed: u64,
    pub disputed: u64,
    pub cancelled: u64,
}

/// Fee collection chart data — bucketed by ledger range.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeChartData {
    pub points: Vec<ChartPoint>,
    pub total_fees: u64,
}

/// Aggregated analytics snapshot for a single user — drives user stats charts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStatsSnapshot {
    pub address: Address,
    pub total_trades: u32,
    pub total_volume: u64,
    pub success_rate_bps: u32,
    pub trades_as_seller: u32,
    pub trades_as_buyer: u32,
}

/// Filter for analytics queries — ledger range only (no PII).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsFilter {
    /// Inclusive lower bound (ledger sequence). None = from genesis.
    pub from_ledger: Option<u32>,
    /// Inclusive upper bound (ledger sequence). None = current ledger.
    pub to_ledger: Option<u32>,
    /// Number of ledgers per bucket for time-series charts (0 = no bucketing).
    pub bucket_size: u32,
}
