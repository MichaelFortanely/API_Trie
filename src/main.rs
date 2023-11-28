mod trie;
use axum::extract::State;

// Use statements for specific items if needed
use crate::trie::*;
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;
use std::sync::RwLock;

use axum::{
    routing::{get, delete, post},
    Router,
    Form,
    extract::{Query, Path},
};



#[tokio::main]
async fn main() {
    //provide a file path or don't provide a file path
    let controller = TrieController::new("s.txt".to_string());
    match controller {
        Ok(controller) => {
            axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
                        .serve(routes_crud(controller).into_make_service())
                        .await
                        .unwrap();

        },
        Err(_) => {
            unreachable!();
        } 
    }
}

fn routes_crud(trie: TrieController) -> Router {
    //all routes except /create, used to create a new trie, expect a uuid
    Router::new().route("/", get(|| async { "Hello to Michael's Trie world!"}))
    .route("/prefix/:id", get(get_prefix_search))
    .route("/wordsearch/:id", get(get_word_search))
    .route("/autocomp/:id", get(get_auto_complete))
    .route("/create", post(post_create_trie))
    .route("/metadata/:id", get(get_trie_metdata))
    .route("/addmany/:id", post(add_multiple_words))
    .route("/deleteword/:id", delete(delete_word))
    .route("/clear/:id", delete(delete_all))
    .route("/deletetrie/:id", delete(delete_trie))
    .route("/addword/:id", post(add_single_word))
    .with_state(trie)
}

//
async fn post_create_trie(State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    // println!("post add_words");
    let mut trie_map = trie_controller.trie_map.write().unwrap();
    let your_new_key = Uuid::new_v4().to_string();
    //will never fail with zero length string
    trie_map.insert(your_new_key.clone(), Arc::new(RwLock::new(match Trie::new("".to_string()).0 {
        Ok(trie) => trie,
        Err(_) => panic!(),
    })));
    axum::response::Json(serde_json::json!({
        "uuid": your_new_key,
    }))
}

async fn delete_trie(Path(word): Path<String>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    // println!("post add_words");
    // println!("Item ID: {:?}\n", word);
    let mut trie_map = trie_controller.trie_map.write().unwrap();
    match trie_map.remove(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let mut trie = trie.write().unwrap();
            let (num_words_before, trie_size_before) = trie.get_metadata();
            trie.delete_dictionary();
            let (num_words_after, trie_size_after) = trie.get_metadata();
            axum::response::Json(serde_json::json!({
                "nodes deleted": format!("{}",  trie_size_before - trie_size_after),
                "words deleted": format!("{}",  num_words_before - num_words_after),
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
struct Params {
    word: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PathParams {
    id: Option<String>,
}

//I want to have 3 different possible outocmes
//1. expected success
//2. error from not includding query param
//3. error from not giving valid input (non letterinput)

// http://localhost:3000/prefix?word=salty
//add params with key-value pairs spearated by ?
fn verify_word_query_param(params: &Params, is_autocomp: bool) -> Result<bool, axum::response::Json<serde_json::Value>>{
    match &params.word {
        Some(w) => {
            if w.len() == 0 && !is_autocomp{
                Err(axum::response::Json(serde_json::json!({
                "error": "empty 'word' parameter in the query"
            })))
        } else{
            Ok(true)
        }
        },
        None => Err(axum::response::Json(serde_json::json!({
            "error": "Missing 'word' parameter in the query"
        })))
    }
}

async fn get_prefix_search(Path(word): Path<String>, Query(params): Query<Params>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    // println!("Item ID: {:?}\n", word);
    let trie_map = trie_controller.trie_map.read().unwrap();
    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let trie = trie.read().unwrap();
            
    // println!("{:?}", params);
    match verify_word_query_param(&params, false) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    // println!("word in prefix search {word}");
    let return_bool = trie.does_prefix_exist(word);
    match return_bool {
        Ok(_) => {
            axum::response::Json(serde_json::json!({
                "is_found": return_bool
            }))
        },
        Err(error) => {
            axum::response::Json(serde_json::json!({
                "error": error
            }))
        }
    }}}
}

//create a RwLock on the hashmap itself, and only need a mutable instance for post method, otherwise just need a read
async fn get_word_search(Path(word): Path<String>, Query(params): Query<Params>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie_map = trie_controller.trie_map.read().unwrap();
    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let trie = trie.read().unwrap();
          
    // println!("{:?}", params);
    match verify_word_query_param(&params, false) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    // println!("word in word search is {word}");
    let return_bool = trie.does_word_exist(word);

    match return_bool {
        Ok(_) => {
            axum::response::Json(serde_json::json!({
                "is_found": return_bool
            }))
        },
        Err(e) => {
            axum::response::Json(serde_json::json!({
                "error": e
            }))
        }
    }
}}}

async fn add_single_word(Path(word): Path<String>, Query(params): Query<Params>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value> {
    let trie_map = trie_controller.trie_map.read().unwrap();
    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let mut trie = trie.write().unwrap();
            
    // println!("{:?}", params);
    match verify_word_query_param(&params, false) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    let trie_size_before = trie.get_metadata().1;
    let result = trie.add_words(vec![word.clone()]);
    match result {
        Ok(_) => {
            axum::response::Json(serde_json::json!({
                word : format!("number of nodes added: {}", trie.get_metadata().1 - trie_size_before)
            }))
        },
        Err(e) => {
            axum::response::Json(serde_json::json!({
                "error": e
            }))
        }
    }}}
}

async fn get_auto_complete(Path(word): Path<String>, Query(params): Query<Params>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie_map = trie_controller.trie_map.read().unwrap();
    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let trie = trie.read().unwrap();
          
    // println!("{:?}", params);
    match verify_word_query_param(&params, true) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    // println!("{word}");
    if word.len() == 0{
        return axum::response::Json(serde_json::json!({"auto_complete": trie.entire_dictionary()}))
    }
    let suggestions = trie.autocomplete(word);
    match suggestions {
        Ok(vec) => axum::response::Json(serde_json::json!({"auto_complete": vec})),
        Err(e) => axum::response::Json(serde_json::json!({"error": e }))
    }
}}
}

async fn get_trie_metdata(Path(word): Path<String>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie_map = trie_controller.trie_map.read().unwrap();
    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let trie = trie.read().unwrap();
          
    //TODO implement error handling for UUID, just trying to get this file to compile for now
    let metadata = trie.get_metadata();
    //return number of words, number of nodes
    axum::response::Json(serde_json::json!({
        "num_words": metadata.0,
        "num_trie_nodes": metadata.1
    }))
}}}

async fn delete_word(Path(word): Path<String>, Query(params): Query<Params>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    // trie_controller.create_new_trie();
    let trie_map = trie_controller.trie_map.read().unwrap();
    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let mut trie = trie.write().unwrap();
            
    // println!("{:?}", params);
    match verify_word_query_param(&params, false) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    match trie.delete_word(word.clone()){
        Ok(possibly_deleted) => axum::response::Json(serde_json::json!({
            word : format!("was deleted?: {}", possibly_deleted)
        })),
        Err(e) => {
            axum::response::Json(serde_json::json!({
                "error": e
            }))
        },
    }}}
}

async fn delete_all(Path(word): Path<String>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie_map = trie_controller.trie_map.read().unwrap();
    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let mut trie = trie.write().unwrap();
            
    let (num_words_before, trie_size_before) = trie.get_metadata();
    trie.delete_dictionary();
    let (num_words_after, trie_size_after) = trie.get_metadata();
    axum::response::Json(serde_json::json!({
        "nodes deleted": format!("{}",  trie_size_before - trie_size_after),
        "words deleted": format!("{}",  num_words_before - num_words_after),
    }))
}}
} 


#[derive(Debug, Deserialize)]
struct AddWordsRequest {
    words: String,
}

async fn add_multiple_words( Path(word): Path<String>, State(trie_controller): State<TrieController>,
    Form(sign_up): Form<AddWordsRequest>
) -> axum::response::Json<serde_json::Value>{
    let words = sign_up.words.lines().map(|s| s.to_string()).collect();
    let trie_map = trie_controller.trie_map.read().unwrap();

    match trie_map.get(&word) {
        None => return axum::response::Json(serde_json::json!({
            "invalid id": word
        })),
        Some(trie) => {
            let mut trie = trie.write().unwrap();
            
            let (num_words_before, trie_size_before) = trie.get_metadata();
            
            let result = trie.add_words(words);
            match result {
                Ok(_) => {
                    let (num_words_after, trie_size_after) = trie.get_metadata();
                    axum::response::Json(serde_json::json!({
                        "nodes added": format!("{}", trie_size_after - trie_size_before),
                        "words added": format!("{}", num_words_after - num_words_before),
                    }))
                },
                Err(e) => {
                    axum::response::Json(serde_json::json!({
                        "error": e
                    }))
                }
            }
        }
    } 
}
