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
            my_trie.add_words(starting_words);
            println!("number of nodes in the trie is {}", my_trie.trie_size);
            println!("number of words in trie is {}", my_trie.num_words);
            let my_str = String::from("BoAt");
            let my_str2 = my_str.clone();
            println!("Does word {my_str} exist: {}", my_trie.does_word_exist(my_str.clone()));
            println!("Does prefix {my_str2} exist: {}", my_trie.does_prefix_exist(my_str2.clone()));
            // my_trie.autocomplete("pepperyGirl".eto_string());
            // println!("{:?}", my_trie.autocomplete(String::from("pepper")));
            let sugegestion = String::from("appalachian");
            let suggestions = my_trie.autocomplete(sugegestion.clone());
            println!("num_suggestions: {}; suffix list for suggestion {}: {:?}",  suggestions.len(), sugegestion.clone(), suggestions);
            println!("entire dictionary: {:?}", my_trie.entire_dictionary());
            println!("successful deletion of {}: {}", sugegestion.clone(), my_trie.delete_word(sugegestion.clone()));
            println!("Does word {} exist: {}", sugegestion.clone(), my_trie.does_word_exist(sugegestion.clone()));
            let auto_comp_after_del = String::from("ap");
            let second_round_suggest =  my_trie.autocomplete(auto_comp_after_del.clone());
            println!("num_suggestions: {}; suffix list for suggestion {}: {:?}",  second_round_suggest.len(), auto_comp_after_del.clone(), second_round_suggest);
            println!("entire dictionary: {:?}", my_trie.entire_dictionary());
            println!("number of nodes in the trie is {}", my_trie.trie_size);
            println!("number of words in trie is {}", my_trie.num_words);
            let third_word: String = String::from("appalachianS");
            my_trie.add_words(vec![third_word.clone()]);
            println!("added word {}", third_word.clone());
            let third_round_suggest =  my_trie.autocomplete(auto_comp_after_del.clone());
            println!("num_suggestions: {}; suffix list for suggestion {}: {:?}",  third_round_suggest.len(), third_word.clone(), third_round_suggest);
            println!("entire dictionary: {:?}", my_trie.entire_dictionary());
            println!("number of nodes in the trie is {}", my_trie.trie_size);
            println!("number of words in trie is {}", my_trie.num_words);
            //TODO figure out why number of nodes in trie is incorrect, everything else looks good tho
            //TODO right unit tests before putting into an API
            println!("deleting entire dictionary");
            my_trie.delete_dictionary();//debug this, figure out why not working
            //should be able to delete all words like this
            println!("number of nodes in the trie is {}", my_trie.trie_size);
            println!("number of words in trie is {}", my_trie.num_words);
        },
        (Err(e), _) => print!("had error {:?}", e),
    }
    
}
//1. taking in new words to add to the trie, 
//2. giving auto complete suggestions, and 
//3. giving a bool for if the word exists in the trie or not
//4. size of the Trie or the number of nodes
//What if I built an application in rust and one in python using my API
//how to implement a trie --> do this first in sync manner and then consider async
