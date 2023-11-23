use std::iter::Peekable;
use std::fs;
use std::collections::HashMap;
use std::str::Chars;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug)]
pub enum CustomError {
    InvalidFormatting,
    InvalidCharacter(char),
    UnableToOpen(std::io::Error)
}

//How to make this multi threaded
#[derive(Debug)]
pub struct TrieNode {
    is_word: bool,
    char_val: char,
    //changed implementation to use Arc from Rc to be able to use multiple threads
    //Had to make use of RwLock in order to have interior mutability of shared reference with is thread-safe,
    //because RefCell does not implement Sync Trait
    //will be making use of Mutex to actually take care of Synchronization, will not rely on synchronization properties of RwLock at all
    children: HashMap<char, Arc<RwLock<TrieNode>>>,
}
//having helper functions that return refernces can get really complicated
//try to implement this without many helper functions i.e. references that return TrieNodes bc then stuff gets really complicated


impl TrieNode {
    fn new(new_char: char) -> Arc<RwLock<Self>>{
        Arc::new(RwLock::new(TrieNode { is_word: false, char_val: new_char, children: HashMap::new() }))
    }

    //in order to return the number of nodes that are deleted, need to bubble that information up the call stack, similar to how _add_words does with Trie caller
    //for return type, first bool is whether the word or prefix (based off of must_be_complete) is found, u32 is num nodes deleted, and last bool
    //tells caller whether to remove child node during a delete operation
    fn _delete_from_trie(&mut self, mut chars: Peekable<Chars>) -> (bool, u32, bool){
        match chars.next() {
            Some(_) => {
                // println!("char is {curr_char} and self.char_val is {}", self.char_val);
                    if let Some(&next_char) = chars.peek() {
                        match self.children.get(&next_char) {
                            Some(value_from_key) => {
                                //next trie node exists
                                let mut return_val =  value_from_key.write().unwrap()._delete_from_trie(chars);
                                if return_val.2 {
                                    //not yet encoutered another word node when bubbling back up call stack -> this assumption was incorrect and 
                                    //caused bug in penultimate commit from this one
                                    //do deletion logic of deleting child
                                    return_val.1 += 1;
                                    drop(self.children.remove(&next_char).unwrap());

                                    //should not be dropping current node if I still have other children
                                    if self.is_word || self.children.keys().len() > 0 {
                                        return_val.2 = false;
                                        //this will tell caller in a delete operation to stop deleting child node
                                    }
                                }
                                return return_val;
                            },
                            //no trienode for next char in iterator
                            None => return (false, 0, false)
                        }
                    } else{
                        //exhaustively matched all chars in iterator to trie
                        if self.is_word {
                            let mut delete_child = false;
                            println!("delete operation initiated");
                            //If I have no children then I need to bubble back up the call stack
                            //until self.is_word equals true and delete all nodes until then
                            //if I have children then just mark myself as not a word anymore
                            self.is_word = false;

                            if self.children.keys().len() == 0 {
                                //need to bubble up the call stack until I find a word, deleting all words until
                                delete_child = true;
                            } //else I need to do nothing
                            //only increment this u32 return value when actually deleting the child
                            //the boolean just tells the caller whether or not to actually delete the node it originall called this function on
                            return (true, 0, delete_child);
                        } else {
                            return (false, 0, false);
                        }
                    }
            },
            None => {
                unreachable!();
            }
        }
    }

    fn _search_tree(&self, mut chars: Peekable<Chars>, must_be_complete: bool, suffic_vec: &mut Vec<String>, get_suffixes: bool) -> bool{
        match chars.next() {
            Some(_) => {
                // println!("char is {curr_char} and self.char_val is {}", self.char_val);
                    if let Some(&next_char) = chars.peek() {
                        match self.children.get(&next_char) {
                            Some(value_from_key) => {
                                //next trie node exists
                                return value_from_key.read().unwrap()._search_tree(chars, must_be_complete, suffic_vec, get_suffixes)
                            },
                            //no trienode for next char in iterator
                            None => return false
                        }
                    } else{
                        //exhaustively matched all chars in iterator to trie
                        if self.is_word && must_be_complete || !must_be_complete {
                            if get_suffixes {
                                println!("function _auto_complete initiated");
                                let mut mut_string = String::new();
                                self._autocomplete(&mut mut_string, suffic_vec, true);
                            }
                            return true;
                        } else {
                            return false;
                        }
                    }
            },
            None => {
                unreachable!();
            }
        }
    }
   

    fn _autocomplete(&self, s: &mut String, suffix_vec: &mut Vec<String>, is_start: bool) {
        //do a DFS from this point  -> add current nodes 
        //I will only need to do the cloning when I am at a word (leaf or node says is word)
        // println!("char here is {}", self.char_val);
        if !is_start{ //included is_start boolean in order to not push ending char of word of interest
            s.push(self.char_val);
        }
        if self.is_word && !is_start{
            // println!("is word: {}", s.clone());
            suffix_vec.push(s.clone());
        }
        for value in self.children.values() {
            value.write().unwrap()._autocomplete(s, suffix_vec, false);
        }
        if !is_start{
            s.pop();
        }
    }

    //u32 is number of new nodes inserted, bool is whether this is a new word
    fn _add_word(&mut self, mut new_word: impl Iterator<Item = char>) -> (u32, bool) {
        if let Some(first_char) = new_word.next() {
            let mut val_to_add = 0;
            if !self.children.contains_key(&first_char) {
                // println!("{first_char} not in nodes children");
                self.children.insert(first_char, TrieNode::new(first_char));
                val_to_add = 1;
            } else{
                // println!("{first_char} exists in node children");
            }
            let returned_val = self.children.get_mut(&first_char).unwrap().write().unwrap()._add_word(new_word);
            return (val_to_add + returned_val.0, returned_val.1);
        } else{
            //if I have reached the end of my iterator then I will declare that TrieNode I have is the end of a word
            if self.is_word == false{
                self.is_word = true;
                // println!("Im at the end of a new word! char_val is {}", {self.char_val});
                return (0, true)
            }
            // println!("This word already exists{}", {self.char_val});
            return (0, false);
        }
    }
}
//base of trie is a trieNode with a value of !
#[derive(Debug)]
pub struct Trie {
    pub base_trie_node: TrieNode,
    pub trie_size: u32,
    pub num_words: u32,
}

impl Trie{
    //tree will take input characters a-zA-Z, but it will be case insensitive for all methods
    //will return an empty trie if string is blank
    //otherwise for each word that is separated by a new line character, it will be added to the trie
    pub fn new(file_path: String) -> (Result<Self, CustomError>, Vec<String>){
        //need to read in characters from file
        let base_trie_node = Trie { base_trie_node: TrieNode { is_word: false, char_val: '!', children: HashMap::new() }, trie_size: 0, num_words: 0 };
        if file_path.len() == 0 {
            return (Ok(base_trie_node), vec![])
        } 
        //I will enforce that all characters must be a-z
        //first thing to do is try read from the file -> will given in the form of form data in body of API
        let mut verified_contents: Vec<String> = Vec::new();
        match fs::read_to_string(file_path) {
            Ok(contents) => {
                let contents = contents.split("\n");
                for content in contents {
                    // println!("file contents: {content} len: {}", content.len());
                    if content.len() == 0 {
                        return (Err(CustomError::InvalidFormatting), vec![])
                    }
                    for indv_char in content.chars() {
                        if !indv_char.is_ascii_alphabetic() {
                            println!("error with {}", indv_char);
                            return (Err(CustomError::InvalidCharacter(indv_char)), vec![]);
                        }
                    }
                    verified_contents.push(content.to_ascii_lowercase().to_string());
                }
            }, Err(e) => return (Err(CustomError::UnableToOpen(e)), vec![]),
        }
        (Ok(base_trie_node), verified_contents)
    }

    pub fn add_words(&mut self, starting_words: Vec<String>){
        //this function also needs to acquire lock
        //this will equal size of tree, but later on when calling for indiviual words
        //the return type of add_word will return interesting information
        let mut num_nodes_added = 0;
        for word in starting_words {
                let returned_tup = self.base_trie_node._add_word(word.to_ascii_lowercase().chars());
                num_nodes_added += returned_tup.0;
                if returned_tup.1 == true {
                    // println!("{word} is a new word");
                    self.num_words += 1;
                } else{
                    // println!("{word} not a new word");
                }
                // println!("num nodes in the tree is {}", num_nodes_added);
            }
        self.trie_size += num_nodes_added;
        // println!("num nodes in the tree is {}", self.trie_size);
    }
    //NOTE -> base of tree is TrieNode with value !
    //this function returns true only if the string is 
    pub fn does_prefix_exist(&self, s: String) -> bool {
        self.base_trie_node._search_tree(("!".to_string() + &(s.to_ascii_lowercase())).chars().peekable(), false, &mut vec![], false)
    }

    //this function returns true if the word has been added to the trie
    pub fn does_word_exist(&self, s: String) -> bool {
        self.base_trie_node._search_tree(("!".to_string() + &(s.to_ascii_lowercase())).chars().peekable(), true, &mut vec![], false)
    }

    //the two functions above have placholder vectors
    //I will do a DFS using a string that will be added to the vector that is passed in as a mutable reference so it does not need to be returned

    //will return with blank if not even a prefix does not work
    pub fn autocomplete(&self, s: String) -> Vec<String>{
        let mut suffix_list: Vec<String> = vec![];
        self.base_trie_node._search_tree(("!".to_string() + &(s.to_ascii_lowercase())).chars().peekable(), false, &mut suffix_list, true);
        suffix_list
    }

    pub fn entire_dictionary(&self) -> Vec<String> {
        self.autocomplete(String::new())
    }

    //word does not exist -> do nothing
    //remember to change trie size and number of nodes
    //word does exist
    //is word a leaf node -> then recursively delete all nodes until you reach a word node
    //is word not a leaf node -> then mark as not a word anymore
    //returns true if deleted, false if not deleted
    pub fn delete_word(&mut self, s: String) -> bool{
        //this function should acquire lock if it does not yet have
        let return_val = self.base_trie_node._delete_from_trie(("!".to_string() + &(s.to_ascii_lowercase())).chars().peekable());
        if return_val.0{
            //at this point I know for sure the word exists
            self.num_words -= 1;
            self.trie_size -= return_val.1;

        } 
        println!("Number of nodes deleted {}", return_val.1);
        return_val.0
    }

    //this function should acquire lock
    pub fn delete_dictionary(&mut self){
        for word in self.entire_dictionary() {
            println!("next word to delete is {}", {word.clone()});
            self.delete_word(word);
            println!("Dictionary after deletion{:?}", self.entire_dictionary());
            println!("trie size: {}; num_words: {}", self.trie_size, self.num_words);
        }
        println!("done!");
    }
    
}
#[derive(Debug, Clone)]
pub struct TrieController{
    //I want to have a mutex on each vector
    trie: Arc<RwLock<Trie>>,
}

//have ModelController control synchronization, start with a single trie controller by RwLock
impl TrieController {
    pub fn new(file_path: String) -> Self {
         match Trie::new(file_path){
            (Ok(mut trie), starting_words) => {
                TrieController {trie: Arc::new(RwLock::new(trie))}
            },
            (Err(e), _) => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //TEST FUNCTIONALITY OF TRIE
    #[test]
    fn create_empty_trie() {
        match Trie::new("".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 0);
                assert_eq!(my_trie.num_words, 0);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn create_trie_from_file() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn invalid_character() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => {if let CustomError::UnableToOpen(inner_error) = e {
                assert_eq!(inner_error.kind(), std::io::ErrorKind::NotFound);
            } else {
                panic!("Expected UnableToOpen error, but got {:?}", e);
            }},
        }
    }

    #[test]
    fn no_new_line() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => {if let CustomError::UnableToOpen(inner_error) = e {
                assert_eq!(inner_error.kind(), std::io::ErrorKind::NotFound);
            } else {
                panic!("Expected UnableToOpen error, but got {:?}", e);
            }},
        }
    }

    #[test]
    fn no_blank_line() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => {if let CustomError::UnableToOpen(inner_error) = e {
                assert_eq!(inner_error.kind(), std::io::ErrorKind::NotFound);
            } else {
                panic!("Expected UnableToOpen error, but got {:?}", e);
            }},
        }
    }

    #[test]
    fn create_trie_from_noexistent_file() {
        match Trie::new("doesNOTexist.txt".to_string()) {
            (Ok(_), _) => {
                panic!();
            }, (Err(e), _) => {if let CustomError::UnableToOpen(inner_error) = e {
                assert_eq!(inner_error.kind(), std::io::ErrorKind::NotFound);
            } else {
                panic!("Expected UnableToOpen error, but got {:?}", e);
            }},
        }
    }

    //TODO
    #[test]
    fn retrieve_all_words() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn delete_all_words() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn add_invalid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn add_uppercase_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn prefix_search_invalid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn prefix_search_valid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn prefix_search_uppercase_valid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn word_search_invalid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn word_search_valid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn word_search_uppercase_valid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn add_all_then_delete_all() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn multithread_delete_then_search() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    ////END TEST FUNCTIONALITY OF TRIE

    //TEST FUNCTIONALITY OF TRIECONTROLLER
    //TODO
    #[test]
    fn multithread_delete_while_search() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //TODO
    #[test]
    fn multithread_read_ops() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }
    
    //TODO
    #[test]
    fn multithread_modify_during_read() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    //END TEST FUNCTIONALITY OF TRIECONTROLLER
}

// fn main() {
//     println!("Hello, world!");
//     // let first = TrieNode::new('a');
//     // println!("{:?}",first);
//     match Trie::new(String::from("s.txt")){
//         (Ok(mut my_trie), starting_words) => {
//             my_trie.add_words(starting_words);
//             println!("number of nodes in the trie is {}", my_trie.trie_size);
//             println!("number of words in trie is {}", my_trie.num_words);
//             let my_str = String::from("BoAt");
//             let my_str2 = my_str.clone();
//             println!("Does word {my_str} exist: {}", my_trie.does_word_exist(my_str.clone()));
//             println!("Does prefix {my_str2} exist: {}", my_trie.does_prefix_exist(my_str2.clone()));
//             // my_trie.autocomplete("pepperyGirl".eto_string());
//             println!("{:?}", my_trie.autocomplete(String::from("")));
//             my_trie.delete_word(String::from("BoAt"));
//             // let sugegestion = String::from("appalachian");
//             // let suggestions = my_trie.autocomplete(sugegestion.clone());
//             // println!("num_suggestions: {}; suffix list for suggestion {}: {:?}",  suggestions.len(), sugegestion.clone(), suggestions);
//             // println!("entire dictionary: {:?}", my_trie.entire_dictionary());
//             // println!("successful deletion of {}: {}", sugegestion.clone(), my_trie.delete_word(sugegestion.clone()));
//             // println!("Does word {} exist: {}", sugegestion.clone(), my_trie.does_word_exist(sugegestion.clone()));
//             // let auto_comp_after_del = String::from("ap");
//             // let second_round_suggest =  my_trie.autocomplete(auto_comp_after_del.clone());
//             // println!("num_suggestions: {}; suffix list for suggestion {}: {:?}",  second_round_suggest.len(), auto_comp_after_del.clone(), second_round_suggest);
//             // println!("entire dictionary: {:?}", my_trie.entire_dictionary());
//             // println!("number of nodes in the trie is {}", my_trie.trie_size);
//             // println!("number of words in trie is {}", my_trie.num_words);
//             // let third_word: String = String::from("appalachianS");
//             // my_trie.add_words(vec![third_word.clone()]);
//             // println!("added word {}", third_word.clone());
//             // let third_round_suggest =  my_trie.autocomplete(auto_comp_after_del.clone());
//             // println!("num_suggestions: {}; suffix list for suggestion {}: {:?}",  third_round_suggest.len(), third_word.clone(), third_round_suggest);
//             // println!("entire dictionary: {:?}", my_trie.entire_dictionary());
//             // println!("number of nodes in the trie is {}", my_trie.trie_size);
//             // println!("number of words in trie is {}", my_trie.num_words);
//             // //TODO figure out why number of nodes in trie is incorrect, everything else looks good tho
//             // //TODO right unit tests before putting into an API
//             // println!("deleting entire dictionary");
//             // my_trie.delete_dictionary();//debug this, figure out why not working
//             // //should be able to delete all words like this
//             // println!("number of nodes in the trie is {}", my_trie.trie_size);
//             // println!("number of words in trie is {}", my_trie.num_words);
//         },
//         (Err(e), _) => print!("had error {:?}", e),
//     }
    
// }
//1. taking in new words to add to the trie, 
//2. giving auto complete suggestions, and 
//3. giving a bool for if the word exists in the trie or not
//4. size of the Trie or the number of nodes
//What if I built an application in rust and one in python using my API
//how to implement a trie --> do this first in sync manner and then consider async


//TODO add tests --> use this driver code to help create tests
