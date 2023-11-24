mod trie;
use axum::extract::State;

// Use statements for specific items if needed
use crate::trie::*;
use serde::Deserialize;

use axum::{
    routing::{get, post},
    Router, response::{IntoResponse, Html}, http::method, extract::{Query, Path},
};



#[tokio::main]
async fn main() {
    //provide a file path or don't provide a file path
    let controller = TrieController::new("".to_string());
    match controller {
        Ok(controller) => {
            axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
                        .serve(routes_crud(axum::extract::State(controller)).into_make_service())
                        .await
                        .unwrap();

        },
        Err(e) => {
            unreachable!();
        } 
    }
}

fn routes_crud(trie: State<TrieController>) -> Router {
    Router::new().route("/", get(|| async { "Hello to Michael's Trie world!"}))
    .route("/prefix", get(get_prefix_search))
    .route("/wordsearch/:name", get(get_word_search))
    .route("/autocomp", get(get_auto_complete))
    .route("/create", post(post_create_trie))
    .with_state(trie)
}

async fn post_create_trie() -> impl IntoResponse{
    println!("patch add_words");
    Html(format!("Hello <strong>Michael</strong>"))
}

#[derive(Debug, Deserialize)]
struct TestParams {
    name: Option<String>,
}

// http://localhost:3000/prefix?name=salty
//add params with key-value pairs spearated by ?
async fn get_prefix_search(Query(params): Query<TestParams>) -> impl IntoResponse{
    println!("{:?}", params);
    //example
    // let name = params.name.as_deref().unwrap_or("JohnSmith");
    // Html(format!("Hello <strong>{name}</strong>"))


}

//example how to get path parms
// async fn get_word_search(Path(name): Path<String>) -> impl IntoResponse{
//     println!("{:?}", name);
//     // let name = name.unwrap_or("JohnSmith");
//     Html(format!("Hello <strong>{name}</strong>"))
// }
async fn get_word_search(Path(name): Path<String>) -> impl IntoResponse{
    println!("{:?}", name);
    // let name = name.unwrap_or("JohnSmith");
    Html(format!("Hello <strong>{name}</strong>"))
}

async fn get_auto_complete() -> impl IntoResponse{
    println!("do_word_search");
    Html("GET automatetion")
}

async fn delete_words() {
    println!("delete_words");
}

//put tests for trie_router here and tests for the trie in trie.rs
// #[cfg(test)]
// mod tests {
//     use super::*;
    
//     #[test]
//     fn name() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
