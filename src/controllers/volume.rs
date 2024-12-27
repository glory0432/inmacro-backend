use crate::{
    dto::request::*,
    utils::{errors::ApiError, jwt::UserClaims},
    AppState,
};
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
pub async fn get_volume_data(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetVolumeDataRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query_builder = QueryBuilder::new("SELECT SUM(total_volume) AS total_quantity, AVG(price) AS average_price, token_symbol, MAX(day_total_volume) as total_volume_day, exchange_id, date_trunc('hour', timestamp)");

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
    query_builder.push(" FROM volume_data WHERE token_symbol=");
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
        .push(" GROUP BY time_interval, exchange_id, token_symbol ORDER BY time_interval ASC");
    let sql_query = query_builder.build();
    let query_result = sql_query.fetch_all(&state.crypto_data_db).await?;
    let mut res: Vec<Vec<String>> = vec![];
    for item in query_result {
        let timestamp = item.get::<DateTime<Utc>, _>("time_interval");
        let exchange_id = item.get::<i32, _>("exchange_id");
        let mut total_volume = item.get::<f64, _>("total_quantity");
        if exchange_id == 1 {
            total_volume = item.get::<f64, _>("total_volume_day");
        }
        let token_symbol: String = item.get::<String, _>("token_symbol");
        let token_parts: Vec<&str> = token_symbol.splitn(2, '-').collect();
        if token_parts.len() != 2 {
            continue;
        }
        let mut unit = 1.0;

        if token_parts[1] == query.unit {
            if exchange_id != 1 {
                unit = item.get::<f64, _>("average_price");
            }
        } else if token_parts[0] == query.unit {
            if exchange_id == 1 {
                unit = 1.0 / item.get::<f64, _>("average_price");
            }
        } else {
            continue;
        }
        let mut res_item: Vec<String> = vec![];
        res_item.push(timestamp.timestamp().to_string());
        res_item.push(exchange_id.to_string());
        res_item.push((total_volume * unit).to_string());
        res.push(res_item);
    }
    return Ok(Json(json!(res)));
}
pub async fn get_24hr_volume_data(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let mut res = vec![];
    for exch_id in 0..=4 {
        let mut volume_quantity = 0.0;
        if exch_id != 1 {
            let total_volume_query = format!("SELECT SUM(total_volume * price) AS volume FROM (SELECT * FROM volume_data WHERE exchange_id = {} ORDER BY timestamp DESC LIMIT 1440) a", exch_id);
            let mut query = QueryBuilder::new(total_volume_query);
            let query = query.build();
            let data = query.fetch_one(&state.crypto_data_db).await?;
            volume_quantity = data.get::<f64, _>("volume");
        } else {
            let total_volume_query = format!("SELECT day_total_volume FROM volume_data WHERE exchange_id = 1 ORDER BY timestamp DESC LIMIT 1");
            let mut query = QueryBuilder::new(total_volume_query);
            let query = query.build();
            let data = query.fetch_one(&state.crypto_data_db).await?;
            volume_quantity = data.get::<f64, _>("day_total_volume");
        }
        let mut query = QueryBuilder::new(format!(
            "SELECT price FROM volume_data WHERE exchange_id = {} ORDER BY timestamp DESC LIMIT 1",
            exch_id
        ));
        let query = query.build();
        let data = query.fetch_one(&state.crypto_data_db).await?;
        let price = data.get::<f64, _>("price");
        let mut res_item = vec![];
        res_item.push(exch_id.to_string());
        res_item.push(volume_quantity.to_string());
        res_item.push(price.to_string());
        res.push(res_item);
    }
    return Ok(Json(json!(res)));
}
