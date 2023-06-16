use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    models::AppState,
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use starknet::{
    core::types::{BlockId, CallFunction, FieldElement},
    macros::selector,
    providers::Provider,
};

#[derive(Deserialize)]
pub struct StarknetIdQuery {
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StarknetIdQuery>,
) -> impl IntoResponse {
    let task_id = 18;
    let addr = &query.addr;

    // get starkname from address
    let call_result = state
        .provider
        .call_contract(
            CallFunction {
                contract_address: state.conf.starknetid_contracts.naming_contract,
                entry_point_selector: selector!("address_to_domain"),
                calldata: vec![*addr],
            },
            BlockId::Latest,
        )
        .await;

    match call_result {
        Ok(result) => {
            let domain_len: u64 = result.result[0].try_into().unwrap();
            if domain_len == 1 {
                // check expiry date
                let expiry_result = state
                    .provider
                    .call_contract(
                        CallFunction {
                            contract_address: state.conf.starknetid_contracts.naming_contract,
                            entry_point_selector: selector!("domain_to_expiry"),
                            calldata: result.result,
                        },
                        BlockId::Latest,
                    )
                    .await;

                match expiry_result {
                    Ok(expiry) => {
                        let expiry_timestamp: u32 = expiry.result[0].try_into().unwrap();
                        let current_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_secs() as u32;
                        if expiry_timestamp >= current_timestamp + 86400 * 365 * 3 {
                            match state.upsert_completed_task(query.addr, task_id).await {
                                Ok(_) => {
                                    (StatusCode::OK, Json(json!({"res": true}))).into_response()
                                }
                                Err(e) => get_error(format!("{}", e)),
                            }
                        } else {
                            get_error("Expiry date is less than 3 years".to_string())
                        }
                    }
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("Invalid domain: subdomains are not eligible".to_string())
            }
        }
        Err(e) => get_error(format!("{}", e)),
    }
}