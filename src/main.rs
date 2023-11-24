mod trie;
use axum::extract::State;

// Use statements for specific items if needed
use crate::trie::*;
use serde::Deserialize;

use axum::{
    routing::{get, delete, post},
    Router, extract::{Query, Path},
};



#[tokio::main]
async fn main() {
    //provide a file path or don't provide a file path
    let controller = TrieController::new("testing_txt_files/input/test1.txt".to_string());
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
    .route("/wordsearch", get(get_word_search))
    .route("/autocomp", get(get_auto_complete))
    // .route("/create", post(post_create_trie))
    .route("/metadata", get(get_trie_metdata))
    .route("/delete", delete(delete_word))
    .route("/add", post(add_single_word))
    .with_state(trie)
}

//TODO if want to allow users to have unique Trie, will need to redesign TrieController to allow for multiple Tries to exist instead of just one
// async fn post_create_trie(State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
//     println!("patch add_words");
//     axum::response::Json(serde_json::json!({
//         "TODO": "post_create_trie"
//     }))
// }

#[derive(Debug, Deserialize)]
struct TestParams {
    word: Option<String>,
}

// http://localhost:3000/prefix?name=salty
//add params with key-value pairs spearated by ?
async fn get_prefix_search(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie = trie_controller.trie.read().unwrap();
    println!("{:?}", params);
    let word = params.word.unwrap_or_default();
    println!("word in prefix search {word}");
    let return_bool = trie.does_prefix_exist(word);
    axum::response::Json(serde_json::json!({
        "is_found": return_bool
    }))
}

//example how to get path parms
// async fn get_word_search(Path(name): Path<String>) -> impl IntoResponse{
//     println!("{:?}", name);
//     // let name = name.unwrap_or("JohnSmith");
//     Html(format!("Hello <strong>{name}</strong>"))
// }
async fn get_word_search(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie = trie_controller.trie.read().unwrap();
    // println!("{:?}", params);
    let word = params.word.unwrap_or_default();
    println!("word in word search is {word}");
    let return_bool = trie.does_word_exist(word);
    axum::response::Json(serde_json::json!({
        "is_found": return_bool
    }))
}

//TODO error handling
async fn add_single_word(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value> {
    let mut trie = trie_controller.trie.write().unwrap();
    let word = params.word.unwrap();
    let trie_size_before = trie.get_metadata().1;
    trie.add_words(vec![word.clone()]);
    println!("delete_words");
    axum::response::Json(serde_json::json!({
        word : format!("number of nodes added: {}", trie.get_metadata().1 - trie_size_before)
    }))
}

//TODO implement various error handling and status codes for methods
async fn get_auto_complete(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    println!("do_word_search");
    let trie = trie_controller.trie.read().unwrap();
    println!("{:?}", params);
    let word = params.word.unwrap_or_default();
    println!("{word}");
    let suggestions = trie.autocomplete(word);
    axum::response::Json(serde_json::json!({
        "auto_complete": suggestions
    }))
}

//TODO need to deal with invalid input coming into API
async fn get_trie_metdata(State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie = trie_controller.trie.read().unwrap();
    let metadata = trie.get_metadata();
    //return number of words, number of nodes
    axum::response::Json(serde_json::json!({
        "num_words": metadata.0,
        "num_trie_nodes": metadata.1
    }))
}

//TODO this might be more informative if returned the number of nodes deleted
async fn delete_word(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let mut trie = trie_controller.trie.write().unwrap();
    let word = params.word.unwrap_or_default();
    let possibly_deleted = trie.delete_word(word.clone());
    println!("delete_words");
    axum::response::Json(serde_json::json!({
        word : format!("deleted: {}", possibly_deleted)
    }))
}
