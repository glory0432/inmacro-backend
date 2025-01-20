use crate::{dto::request::*, utils::errors::ApiError, AppState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use serde_json::json;
use sqlx::{QueryBuilder, Row};
use std::sync::Arc;
use tracing::info;
pub async fn get_balance_data(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetBalanceDataRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query_builder = QueryBuilder::new("SELECT wallet_balance, transfer_balance, token_symbol, exchange_id, date_trunc('hour', timestamp)");

    let mut start_time = Utc::now();
    let mut now_time = start_time.clone();
    match query.interval.as_str() {
        "1D" => {
            start_time = start_time - Duration::days(1);
            query_builder.push(" + INTERVAL '1 minute' * (floor(EXTRACT(MINUTE FROM timestamp) / 5) * 5) AS time_interval");
        }
        "7D" => {
            start_time = start_time - Duration::days(7);
            query_builder.push(" + INTERVAL '1 minute' * (floor(EXTRACT(MINUTE FROM timestamp) / 5) * 5) AS time_interval");
        }
        "1M" => {
            let naive_date = start_time.naive_utc().date();
            let year = naive_date.year();
            let month = naive_date.month();
            let new_month = if month == 1 { 12 } else { month - 1 };
            let new_year = if month == 1 { year - 1 } else { year };
            let month_days = (Utc.ymd(year, month, 1) - Duration::hours(24)).day();
            let mut new_day = naive_date.day();
            if new_day > month_days {
                new_day = month_days;
            }
            start_time = Utc.ymd(new_year, new_month, new_day).and_hms(0, 0, 0);
            query_builder.push(" AS time_interval");
        }
        "1Y" => {
            let naive_date = start_time.naive_utc().date();

            let month = if naive_date.month() + 1 == 13 {
                1
            } else {
                naive_date.month()
            };
            let year = if month == 1 {
                naive_date.year()
            } else {
                naive_date.year() - 1
            };
            let month_days = (Utc.ymd(year, month, 1) - Duration::hours(24)).day();
            let mut day = naive_date.day();
            if day > month_days {
                day = month_days;
            }
            start_time = Utc
                .ymd(naive_date.year() - 1, naive_date.month(), day)
                .and_hms(0, 0, 0);
            query_builder.push(" AS time_interval");
        }
        "All" => {
            query_builder.push(" AS time_interval");
        }
        _ => {
            return Ok(Json(json!([])));
        }
    }
    query_builder.push(" FROM balance_data WHERE token_symbol=");
    query_builder.push_bind(query.symbol);
    if let Some(exchange_id) = query.exchange_id {
        query_builder.push(" AND exchange_id=");
        query_builder.push_bind(exchange_id);
    }
    if start_time != now_time {
        query_builder.push(" AND timestamp>=");
        query_builder.push_bind(start_time);
    }
    query_builder
        .push(" GROUP BY time_interval, exchange_id, token_symbol, wallet_balance, transfer_balance ORDER BY time_interval ASC");
    let sql_query = query_builder.build();
    let query_result = sql_query.fetch_all(&state.crypto_data_db).await?;
    let mut res: Vec<Vec<String>> = vec![];
    for item in query_result {
        let timestamp = item.get::<DateTime<Utc>, _>("time_interval");
        let exchange_id = item.get::<i32, _>("exchange_id");
        let token = item.get::<String, _>("token_symbol");
        let wallet_balance = item.get::<f64, _>("wallet_balance");
        let transfer_balance = item.get::<f64, _>("transfer_balance");
        let mut res_item: Vec<String> = vec![];
        res_item.push(timestamp.timestamp().to_string());
        res_item.push(exchange_id.to_string());
        res_item.push(token);
        res_item.push(wallet_balance.to_string());
        res_item.push(transfer_balance.to_string());
        res.push(res_item);
    }
    return Ok(Json(json!(res)));
}
pub async fn get_latest_balance_data(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetLatestBalanceDataRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query_builder = QueryBuilder::new(format!("SELECT * FROM balance_data WHERE exchange_id = {} AND timestamp = ( SELECT MAX(timestamp) FROM balance_data WHERE exchange_id = {} )", query.exchange_id, query.exchange_id));
    let sql_query = query_builder.build();
    let query_result = sql_query.fetch_all(&state.crypto_data_db).await?;
    let mut res: Vec<Vec<String>> = vec![];
    for item in query_result {
        let timestamp = item.get::<DateTime<Utc>, _>("timestamp");
        let exchange_id = item.get::<i32, _>("exchange_id");
        let token = item.get::<String, _>("token_symbol");
        let wallet_balance = item.get::<f64, _>("wallet_balance");
        let transfer_balance = item.get::<f64, _>("transfer_balance");
        let mut res_item: Vec<String> = vec![];
        res_item.push(timestamp.timestamp().to_string());
        res_item.push(exchange_id.to_string());
        res_item.push(token);
        res_item.push(wallet_balance.to_string());
        res_item.push(transfer_balance.to_string());
        res.push(res_item);
    }
    return Ok(Json(json!(res)));
}
