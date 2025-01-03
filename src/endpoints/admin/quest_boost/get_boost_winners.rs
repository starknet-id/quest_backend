use crate::middleware::auth::auth_middleware;
use crate::models::{AppState, BoostTable};
use crate::utils::get_error;
use axum::{
    extract::{Extension, Query, State},
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetBoostWinnersParams {
    boost_id: i64,
}

#[route(get, "/admin/boosts/get_boost_winners", auth_middleware)]
pub async fn get_boost_winners_handler(
    State(state): State<Arc<AppState>>,
    Extension(_sub): Extension<String>,
    Query(params): Query<GetBoostWinnersParams>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");

    let filter = doc! { "id": params.boost_id };

    match collection.find_one(filter, None).await {
        Ok(Some(boost_doc)) => Json(json!({ "winners": boost_doc.winner })).into_response(),
        Ok(None) => get_error(format!("Boost with id {} not found", params.boost_id)),
        Err(e) => get_error(format!("Error fetching boost winners: {}", e)),
    }
}
