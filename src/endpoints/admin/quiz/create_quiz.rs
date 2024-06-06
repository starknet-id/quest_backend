use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use mongodb::options::{FindOneOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use crate::models::{QuestTaskDocument, QuizInsertDocument};

pub_struct!(Deserialize; CreateQuiz {
    name: String,
    desc: String,
    help_link: String,
    cta: String,
    intro: String,
    quest_id: i32,
});

#[route(post, "/admin/tasks/quiz/create", crate::endpoints::admin::quiz::create_quiz)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateQuiz>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_quiz_doc = &quiz_collection.find_one(last_id_filter.clone(), options.clone()).await.unwrap();

    let mut next_quiz_id = 1;
    if let Some(doc) = last_quiz_doc {
        let last_id = doc.id;
        next_quiz_id = last_id + 1;
    }

    let new_quiz_document = QuizInsertDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        id: next_quiz_id.clone(),
        intro: body.intro.clone(),
    };

    match quiz_collection
        .insert_one(new_quiz_document, None)
        .await
    {
        Ok(res) => res,
        Err(_e) => return get_error("Error creating quiz".to_string()),
    };


    let last_task_doc = &tasks_collection.find_one(last_id_filter.clone(), options.clone()).await.unwrap();
    let mut next_id = 1;
    if let Some(doc) = last_task_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        href: body.help_link.clone(),
        cta: body.cta.clone(),
        quest_id: body.quest_id.clone() as u32,
        id: next_id.clone(),
        verify_endpoint: "/quests/verify_quiz".to_string(),
        verify_endpoint_type: "quiz".to_string(),
        quiz_name: Some(next_quiz_id.clone() as i64),
        task_type: Some("quiz".to_string()),
        discord_guild_id: None,
        verify_redirect: None,

    };

    return match tasks_collection
        .insert_one(new_document, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"id": &next_quiz_id })).into_response(),
        )
            .into_response(),
        Err(_e) => return get_error("Error creating quiz".to_string()),
    };
}