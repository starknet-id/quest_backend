use crate::middleware::auth::auth_middleware;
use crate::models::QuestTaskDocument;
use crate::utils::verify_task_auth;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateCustom {
    id: i64,
    name: Option<String>,
    desc: Option<String>,
    cta: Option<String>,
    verify_endpoint: Option<String>,
    verify_endpoint_type: Option<String>,
    verify_redirect: Option<String>,
    href: Option<String>,
});

#[route(post, "/admin/tasks/custom/update", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<CreateCustom>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let res = verify_task_auth(sub, &collection, &(body.id as i32)).await;
    if !res {
        return get_error("Error updating tasks".to_string());
    }

    // filter to get existing quest
    let filter = doc! {
        "id": &body.id,
    };

    let mut update_doc = doc! {};

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }
    if let Some(href) = &body.href {
        update_doc.insert("href", href);
    }
    if let Some(cta) = &body.cta {
        update_doc.insert("cta", cta);
    }
    if let Some(verify_redirect) = &body.verify_redirect {
        update_doc.insert("verify_redirect", verify_redirect);
    }
    if let Some(verify_endpoint) = &body.verify_endpoint {
        update_doc.insert("verify_endpoint", verify_endpoint);
    }
    if let Some(verify_endpoint_type) = &body.verify_endpoint_type {
        update_doc.insert("verify_endpoint_type", verify_endpoint_type);
    }

    // update quest query
    let update = doc! {
        "$set": update_doc
    };

    // insert document to boost collection
    return match collection.find_one_and_update(filter, update, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task updated successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error updating tasks".to_string()),
    };
}
