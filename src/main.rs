//define an enum that has custom error types
mod trie;

// Use statements for specific items if needed
use crate::trie::trie::*;

fn main() {
    println!("Hello, world!");
    // let first = TrieNode::new('a');
    // println!("{:?}",first);
    match Trie::new(String::from("s.txt")){
        (Ok(mut my_trie), starting_words) => {
            my_trie.add_starting_words(starting_words);
            println!("number of nodes in the trie is {}", my_trie.trie_size);
        },
        (Err(e), _) => print!("had error {:?}", e),
    }
}
//1. taking in new words to add to the trie, 
//2. giving auto complete suggestions, and 
//3. giving a bool for if the word exists in the trie or not
//4. size of the Trie or the number of nodes
//how to implement a trie --> do this first in sync manner and then consider async
