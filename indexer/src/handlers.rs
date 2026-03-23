use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    response::Response,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{EventQuery, ReplayRequest, WebSocketMessage};
use crate::websocket::WebSocketManager;
use crate::{database::Database, models::Event};

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn get_events(
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, AppError> {
    let query = EventQuery {
        limit: params.limit.or(Some(50)),
        offset: params.offset.or(Some(0)),
        ..params
    };

    let events = state.database.get_events(&query).await?;
    Ok(Json(events))
}

pub async fn get_event_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Event>, AppError> {
    let event = state.database.get_event_by_id(id).await?;
    Ok(Json(event))
}

pub async fn get_events_by_trade_id(
    Path(trade_id): Path<u64>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, AppError> {
    let query = EventQuery {
        trade_id: Some(trade_id),
        limit: params.limit.or(Some(50)),
        offset: params.offset.or(Some(0)),
        ..params
    };

    let events = state.database.get_events(&query).await?;
    Ok(Json(events))
}

pub async fn get_events_by_type(
    Path(event_type): Path<String>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, AppError> {
    let query = EventQuery {
        event_type: Some(event_type),
        limit: params.limit.or(Some(50)),
        offset: params.offset.or(Some(0)),
        ..params
    };

    let events = state.database.get_events(&query).await?;
    Ok(Json(events))
}

pub async fn replay_events(
    State(state): State<AppState>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<Vec<Event>>, AppError> {
    let to_ledger = request.to_ledger.unwrap_or(i64::MAX);
    let events = state.database.get_events_in_range(request.from_ledger, to_ledger, "contract_id").await?;

    // Broadcast replayed events to WebSocket clients
    for event in &events {
        let ws_message = WebSocketMessage {
            event_type: event.event_type.clone(),
            data: event.data.clone(),
            timestamp: event.timestamp,
        };
        state.ws_manager.broadcast(ws_message).await;
    }

    Ok(Json(events))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| state.ws_manager.handle_connection(socket))
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub ws_manager: Arc<WebSocketManager>,
}