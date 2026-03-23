-- Create events table
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    contract_id VARCHAR(100) NOT NULL,
    ledger BIGINT NOT NULL,
    transaction_hash VARCHAR(100) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_events_contract_ledger ON events (contract_id, ledger DESC);
CREATE INDEX IF NOT EXISTS idx_events_type ON events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_events_trade_id ON events ((data->>'trade_id')) WHERE data->>'trade_id' IS NOT NULL;