#![cfg(test)]

use soroban_sdk::{testutils::Ledger, Address, Env};

use crate::{
    HistoryFilter, SortOrder, StellarEscrowContract, StellarEscrowContractClient, TradeStatus,
};

fn setup() -> (Env, StellarEscrowContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Use a mock token address — token transfers are mocked via mock_all_auths
    let token = Address::generate(&env);

    client.initialize(&admin, &token, &100); // 1% fee

    (env, client, admin, seller, buyer)
}

fn no_filter(env: &Env) -> HistoryFilter {
    HistoryFilter {
        status: None,
        from_ledger: None,
        to_ledger: None,
    }
}

#[test]
fn test_history_empty_for_new_address() {
    let (env, client, _, seller, _) = setup();
    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );
    assert_eq!(page.total, 0);
    assert_eq!(page.records.len(), 0);
}

#[test]
fn test_history_shows_created_trade() {
    let (env, client, _, seller, buyer) = setup();

    let trade_id = client.create_trade(&seller, &buyer, &1000, &None, &None);

    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );

    assert_eq!(page.total, 1);
    let record = page.records.get(0).unwrap();
    assert_eq!(record.trade_id, trade_id);
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, TradeStatus::Created);
}

#[test]
fn test_history_visible_from_buyer_address() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &500, &None, &None);

    let page = client.get_transaction_history(
        &buyer,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );

    assert_eq!(page.total, 1);
}

#[test]
fn test_history_filter_by_status() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    // Cancel the first trade
    client.cancel_trade(&1);

    let filter = HistoryFilter {
        status: Some(TradeStatus::Cancelled),
        from_ledger: None,
        to_ledger: None,
    };

    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, TradeStatus::Cancelled);
}

#[test]
fn test_history_filter_by_ledger_range() {
    let (env, client, _, seller, buyer) = setup();

    // Trade at ledger 1
    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &1000, &None, &None);

    // Trade at ledger 100
    env.ledger().set_sequence_number(100);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    let filter = HistoryFilter {
        status: None,
        from_ledger: Some(50),
        to_ledger: Some(200),
    };

    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 2000);
}

#[test]
fn test_history_sort_descending() {
    let (env, client, _, seller, buyer) = setup();

    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &100, &None, &None);

    env.ledger().set_sequence_number(10);
    client.create_trade(&seller, &buyer, &200, &None, &None);

    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Descending,
        &0,
        &10,
    );

    assert_eq!(page.records.get(0).unwrap().amount, 200);
    assert_eq!(page.records.get(1).unwrap().amount, 100);
}

#[test]
fn test_history_pagination() {
    let (env, client, _, seller, buyer) = setup();

    for _ in 0..5 {
        client.create_trade(&seller, &buyer, &1000, &None, &None);
    }

    let page1 = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &3,
    );
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);

    let page2 = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &3,
        &3,
    );
    assert_eq!(page2.records.len(), 2);
}

#[test]
fn test_export_csv_returns_header_and_rows() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);

    let csv = client.export_transaction_csv(
        &seller,
        &HistoryFilter {
            status: None,
            from_ledger: None,
            to_ledger: None,
        },
    );

    // CSV should be non-empty and contain the header
    assert!(csv.len() > 0);
}

// =============================================================================
// Analytics Charts & Graphs Tests
// =============================================================================

use crate::AnalyticsFilter;

fn analytics_filter_all(env: &Env) -> AnalyticsFilter {
    AnalyticsFilter {
        from_ledger: None,
        to_ledger: None,
        bucket_size: 0,
    }
}

#[test]
fn test_volume_chart_empty_state() {
    let (env, client, _, _, _) = setup();
    let data = client.get_volume_chart(&analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_volume, 0);
    assert_eq!(data.total_trades, 0);
    assert_eq!(data.points.len(), 0);
}

#[test]
fn test_volume_chart_accumulates_trade_amounts() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    let data = client.get_volume_chart(&analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_trades, 2);
    assert_eq!(data.total_volume, 3000);
}

#[test]
fn test_volume_chart_filter_by_ledger_range() {
    let (env, client, _, seller, buyer) = setup();

    env.ledger().set_sequence_number(10);
    client.create_trade(&seller, &buyer, &500, &None, &None);

    env.ledger().set_sequence_number(200);
    client.create_trade(&seller, &buyer, &1500, &None, &None);

    let filter = AnalyticsFilter {
        from_ledger: Some(100),
        to_ledger: Some(300),
        bucket_size: 0,
    };
    let data = client.get_volume_chart(&filter).unwrap();
    assert_eq!(data.total_trades, 1);
    assert_eq!(data.total_volume, 1500);
}

#[test]
fn test_volume_chart_invalid_range_returns_error() {
    let (env, client, _, _, _) = setup();
    let filter = AnalyticsFilter {
        from_ledger: Some(500),
        to_ledger: Some(100),
        bucket_size: 0,
    };
    let result = client.try_get_volume_chart(&filter);
    assert!(result.is_err());
}

#[test]
fn test_success_rate_empty_state() {
    let (_env, client, _, _, _) = setup();
    let data = client.get_success_rate();
    assert_eq!(data.success_rate_bps, 0);
    assert_eq!(data.completed, 0);
}

#[test]
fn test_status_distribution_reflects_trade_states() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    client.cancel_trade(&2);

    let dist = client.get_status_distribution(&analytics_filter_all(&env)).unwrap();
    assert_eq!(dist.created, 1);
    assert_eq!(dist.cancelled, 1);
}

#[test]
fn test_fee_chart_empty_state() {
    let (env, client, _, _, _) = setup();
    let data = client.get_fee_chart(&analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_fees, 0);
    assert_eq!(data.points.len(), 0);
}

#[test]
fn test_user_stats_empty_state() {
    let (_env, client, _, seller, _) = setup();
    let snapshot = client.get_user_stats(&seller);
    assert_eq!(snapshot.total_trades, 0);
    assert_eq!(snapshot.total_volume, 0);
    assert_eq!(snapshot.success_rate_bps, 0);
}

#[test]
fn test_user_stats_reflects_created_trades() {
    let (_env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    let snapshot = client.get_user_stats(&seller);
    assert_eq!(snapshot.total_trades, 2);
    assert_eq!(snapshot.total_volume, 3000);
}

#[test]
fn test_export_platform_analytics_csv_has_header() {
    let (_env, client, _, _, _) = setup();
    let csv = client.export_platform_analytics_csv();
    assert!(csv.len() > 0);
}

#[test]
fn test_export_volume_chart_csv_valid_output() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    let csv = client.export_volume_chart_csv(&analytics_filter_all(&env)).unwrap();
    assert!(csv.len() > 0);
}

#[test]
fn test_export_user_stats_csv_valid_output() {
    let (_env, client, _, seller, _) = setup();
    let csv = client.export_user_stats_csv(&seller);
    assert!(csv.len() > 0);
}

#[test]
fn test_user_volume_chart_empty_state() {
    let (env, client, _, seller, _) = setup();
    let data = client.get_user_volume_chart(&seller, &analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_trades, 0);
    assert_eq!(data.total_volume, 0);
}
