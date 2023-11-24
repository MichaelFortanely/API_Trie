mod trie;
use axum::extract::State;

// Use statements for specific items if needed
use crate::trie::*;
use serde::Deserialize;

use axum::{
    routing::{get, post, patch},
    Router, response::{IntoResponse, Html}, http::method, extract::{Query, Path},
};



#[tokio::main]
async fn main() {
    //provide a file path or don't provide a file path
    let controller = TrieController::new("testing_txt_files/input/test3.txt".to_string());
    match controller {
        Ok(controller) => {
            axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
                        .serve(routes_crud(controller).into_make_service())
                        .await
                        .unwrap();

        },
        Err(e) => {
            unreachable!();
        } 
    }
}

fn routes_crud(trie: TrieController) -> Router {
    Router::new().route("/", get(|| async { "Hello to Michael's Trie world!"}))
    .route("/prefix", get(get_prefix_search))
    .route("/wordsearch/:name", get(get_word_search))
    .route("/autocomp", get(get_auto_complete))
    .route("/create", post(post_create_trie))
    .route("/metadata", get(get_trie_metdata))
    .route("/delete", patch(delete_words))
    .with_state(trie)
}

async fn post_create_trie(State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    println!("patch add_words");
    axum::response::Json(serde_json::json!({
        "TODO": "post_create_trie"
    }))
}

#[derive(Debug, Deserialize)]
struct TestParams {
    name: Option<String>,
}

// http://localhost:3000/prefix?name=salty
//add params with key-value pairs spearated by ?
async fn get_prefix_search(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    // println!("{:?}", params);
    // let name = params.name.unwrap_or_default();
    // println!("{name}");
    axum::response::Json(serde_json::json!({
        "TODO": "get_prefix_search"
    }))


}

//example how to get path parms
// async fn get_word_search(Path(name): Path<String>) -> impl IntoResponse{
//     println!("{:?}", name);
//     // let name = name.unwrap_or("JohnSmith");
//     Html(format!("Hello <strong>{name}</strong>"))
// }
async fn get_word_search(Path(name): Path<String>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    println!("{:?}", name);
    // let name = name.unwrap_or("JohnSmith");
    axum::response::Json(serde_json::json!({
        "TODO": "get_word_search"
    }))
}

async fn get_auto_complete(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    println!("do_word_search");
    let trie = trie_controller.trie.read().unwrap();
    // let metadata = trie.autocomplete();
    axum::response::Json(serde_json::json!({
        "TODO": "get_auto_complete"
    }))
}

async fn get_trie_metdata(State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie = trie_controller.trie.read().unwrap();
    let metadata = trie.get_metadata();
    //return number of words, number of nodes
    axum::response::Json(serde_json::json!({
        "num_words": metadata.0,
        "num_trie_nodes": metadata.1
    }))
}

async fn delete_words(State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    println!("delete_words");
    axum::response::Json(serde_json::json!({
        "TODO": "delete_words"
    }))
}
