//define an enum that has custom error types
mod trie;

// Use statements for specific items if needed
use crate::trie::*;
use serde::Deserialize;

use axum::{
    routing::{get, post},
    Router, response::{IntoResponse, Html}, http::method, extract::{Query, Path},
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    //DELETE ME LATER
    let mut myTrie = Trie::new(String::from("s.txt"));
    match myTrie{
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                println!("number of nodes in the trie is {}", my_trie.trie_size);
                println!("number of words in trie is {}", my_trie.num_words);
                let my_str = String::from("BoAt");
                let my_str2 = my_str.clone();
                println!("Does word {my_str} exist: {}", my_trie.does_word_exist(my_str.clone()));
                println!("Does prefix {my_str2} exist: {}", my_trie.does_prefix_exist(my_str2.clone()));
            },
            (Err(e), _) => print!("had error {:?}", e),
        }
    //DELETE ABOVE LATER
    let app = Router::new().route("/", get(|| async { "Hello to Michael's Trie world!"}))
    .route("/prefix", get(get_prefix_search))
    .route("/wordsearch/:name", get(get_word_search))
    .route("/autocomp", get(get_auto_complete))
    .route("/create", post(post_create_trie));
    // .route("/new_trie", post(post_create_trie()));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
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
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn name() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
