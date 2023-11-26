mod trie;
use axum::extract::State;

// Use statements for specific items if needed
use crate::trie::*;
use serde::Deserialize;

use axum::{
    routing::{get, delete, post},
    Router,
    Form,
    extract::Query,
};
//TODO JSON parsing to add multiple words, Postman API docs, user authentication



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
    .route("/addmany", post(add_multiple_words))
    .route("/delete", delete(delete_word))
    // .route("/clear", delete(delete_word)) //TODO deleteALL
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

//I want to have 3 different possible outocmes
//1. expected success
//2. error from not includding query param
//3. error from not giving valid input (non letterinput)

// http://localhost:3000/prefix?word=salty
//add params with key-value pairs spearated by ?
fn verify_word_query_param(params: &TestParams, is_autocomp: bool) -> Result<bool, axum::response::Json<serde_json::Value>>{
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

async fn get_prefix_search(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie = trie_controller.trie.read().unwrap();
    println!("{:?}", params);
    match verify_word_query_param(&params, false) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    println!("word in prefix search {word}");
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
    }
}

async fn get_word_search(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie = trie_controller.trie.read().unwrap();
    println!("{:?}", params);
    match verify_word_query_param(&params, false) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    println!("word in word search is {word}");
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
}

async fn add_single_word(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value> {
    let mut trie = trie_controller.trie.write().unwrap();
    println!("{:?}", params);
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
    }
}

async fn get_auto_complete(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let trie = trie_controller.trie.read().unwrap();
    println!("{:?}", params);
    match verify_word_query_param(&params, true) {
        Ok(_) => {},
        Err(e) => return e,
    }
    //confirmed word exists with non zero length
    let word = params.word.unwrap();
    println!("{word}");
    if word.len() == 0{
        return axum::response::Json(serde_json::json!({"auto_complete": trie.entire_dictionary()}))
    }
    let suggestions = trie.autocomplete(word);
    match suggestions {
        Ok(vec) => axum::response::Json(serde_json::json!({"auto_complete": vec})),
        Err(e) => axum::response::Json(serde_json::json!({"error": e }))
    }
    
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

async fn delete_word(Query(params): Query<TestParams>, State(trie_controller): State<TrieController>) -> axum::response::Json<serde_json::Value>{
    let mut trie = trie_controller.trie.write().unwrap();
    println!("{:?}", params);
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
    }
    
}
#[derive(Debug, Deserialize)]
struct AddWordsRequest {
    words: String,
}

async fn add_multiple_words( 
    State(trie_controller): State<TrieController>,
    Form(sign_up): Form<AddWordsRequest>
) -> axum::response::Json<serde_json::Value>{
    // println!("sign_up is {:?}", sign_up);
    let mut words = sign_up.words.lines().map(|s| s.to_string()).collect();
    // println!("words are {:?}", words);
    
    let mut trie = trie_controller.trie.write().unwrap();
   
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
