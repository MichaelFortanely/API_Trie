use std::iter::Peekable;
use std::fs;
use std::collections::HashMap;
use std::str::Chars;
use std::sync::Arc;
use std::sync::RwLock;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub enum CustomError {
    InvalidFormatting,
    //invalid char in the string
    InvalidCharacter(char, String),
    UnableToOpen
}

//How to make this multi threaded
#[derive(Debug)]
pub struct TrieNode {
    is_word: bool,
    char_val: char,
    //changed implementation to use Arc from Rc to be able to use multiple threads
    //Had to make use of RwLock in order to have interior mutability of shared reference with is thread-safe,
    //because RefCell does not implement Sync Trait
    //will be making use of a single RwLock at the TrieController level (one level above the Trie Level)
    //to actually take care of Synchronization, will not rely on synchronization properties of RwLock at all
    children: HashMap<char, Arc<RwLock<TrieNode>>>,
}


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
                            // println!("delete operation initiated");
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

    fn _search_tree(&self, mut chars: Peekable<Chars>, must_be_complete: bool, suffic_vec: &mut Vec<String>, get_suffixes: bool, is_prefix_search: bool) -> bool{
        match chars.next() {
            Some(_) => {
                // println!("char is {curr_char} and self.char_val is {}", self.char_val);
                    if let Some(&next_char) = chars.peek() {
                        match self.children.get(&next_char) {
                            Some(value_from_key) => {
                                //next trie node exists
                                return value_from_key.read().unwrap()._search_tree(chars, must_be_complete, suffic_vec, get_suffixes, is_prefix_search)
                            },
                            //no trienode for next char in iterator
                            None => return false
                        }
                    } else{
                        //exhaustively matched all chars in iterator to trie
                        if self.is_word && must_be_complete || !must_be_complete {
                            if is_prefix_search {
                                // println!("hereee: !self.is_word: {}", !self.is_word);
                                return !self.is_word
                            }
                            if get_suffixes {
                                // println!("function _auto_complete initiated");
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
    base_trie_node: TrieNode,
    trie_size: u32,
    num_words: u32,
}

//modify the string in place in the vector to lowercase if valid input, otherwise return error
//placed outside of Trie class so Trie::new could use this method
fn validate_string(contents: &mut Vec<String>) -> Result<bool, CustomError>{
    for content in contents.iter_mut() {
        // println!("{}", content.clone());
        // println!("file contents: {content} len: {}", content.len());
        if content.len() == 0 {
            return Err(CustomError::InvalidFormatting)
        }
        for indv_char in content.clone().chars() {
            if !indv_char.is_ascii_alphabetic() {
                // println!("error with {}", indv_char);
                return Err(CustomError::InvalidCharacter(indv_char, content.to_string()));
            }
        }
        //make lowercase
        *content = content.to_ascii_lowercase();
    }
    Ok(true)
}
impl Trie{
    //only accepts non-zero lengths strings with characters a-z orA-Z
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
        match fs::read_to_string(file_path) {
            Ok(contents) => {
                let mut contents: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
                match validate_string(&mut contents) {
                    Ok(_) => return (Ok(base_trie_node), contents),
                    Err(e) => return (Err(e), vec![]),
                }
            }, Err(e) => return (Err(CustomError::UnableToOpen), vec![]),
        }
    }

    pub fn get_metadata(&self) -> (u32, u32){
        return (self.num_words, self.trie_size);
    }

    pub fn add_words(&mut self, mut starting_words: Vec<String>) -> Result<bool, CustomError>{
        //this will equal size of tree, but later on when calling for indiviual words
        //the return type of add_word will return interesting information
            match  validate_string(&mut starting_words){
                Ok(_) => {
                let mut num_nodes_added = 0;
                for word in starting_words {
                    let returned_tup = self.base_trie_node._add_word(word.chars());
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
                Ok(true)
            }, Err(e) => Err(e),
        }
        // println!("num nodes in the tree is {}", self.trie_size);
    }
    //NOTE -> base of tree is TrieNode with value !
    //this function returns true only if the string is 
    //gonna need to change all of these to return custom errors
    pub fn does_prefix_exist(&self, s: String) -> Result<bool, CustomError> {
        let mut v = vec![s];
        match validate_string(&mut v) {
            Ok(_) => Ok(self.base_trie_node._search_tree(("!".to_string() + &v[0]).chars().peekable(), false, &mut vec![], false, true)),
            Err(e) => Err(e),
        }
    }

    //this function returns true if the word has been added to the trie
    //returning error for invalidly formatted words now
    pub fn does_word_exist(&self, s: String) -> Result<bool, CustomError> {
        let mut v = vec![s];
        match validate_string(&mut v) {
            Ok(_) => Ok(self.base_trie_node._search_tree(("!".to_string() + &v[0]).chars().peekable(), true, &mut vec![], false, false)),
            Err(e) => Err(e),
        }
    }

    //the two functions above have placholder vectors
    //I will do a DFS using a string that will be added to the vector that is passed in as a mutable reference so it does not need to be returned

    //will return with blank if not even a prefix does not work
    pub fn autocomplete(&self, s: String) -> Result<Vec<String>, CustomError>{
        let mut v = vec![s];
        match validate_string(&mut v) {
            Ok(_) => {
        let mut suffix_list: Vec<String> = vec![];
        self.base_trie_node._search_tree(("!".to_string() + &v[0]).chars().peekable(), false, &mut suffix_list, true, false);
        Ok(suffix_list)
    },
    Err(e) => return Err(e),
}
    }

    pub fn entire_dictionary(&self) -> Vec<String> {
        let mut suffix_list: Vec<String> = vec![];
        self.base_trie_node._search_tree("!".to_string().chars().peekable(), false, &mut suffix_list, true, false);
        suffix_list
    }

    //word does not exist -> do nothing
    //remember to change trie size and number of nodes
    //word does exist
    //is word a leaf node -> then recursively delete all nodes until you reach a word node
    //is word not a leaf node -> then mark as not a word anymore
    //returns true if deleted, false if not deleted
    pub fn delete_word(&mut self, s: String) -> Result<bool, CustomError>{
        let mut v = vec![s];
        match validate_string(&mut v) {
        Ok(_) => {
            let return_val = self.base_trie_node._delete_from_trie(("!".to_string() + &v[0]).chars().peekable());
            if return_val.0{
                //at this point I know for sure the word exists
                self.num_words -= 1;
                self.trie_size -= return_val.1;
            } 
            // println!("Number of nodes deleted {}", return_val.1);
            Ok(return_val.0)
        }, Err(e) => Err(e)
    }
    }

    //this function should acquire lock
    pub fn delete_dictionary(&mut self){
        for word in self.entire_dictionary() {
            // println!("next word to delete is {}", {word.clone()});
            self.delete_word(word);
            // println!("Dictionary after deletion{:?}", self.entire_dictionary());
            // println!("trie size: {}; num_words: {}", self.trie_size, self.num_words);
        }
        // println!("done!");
    }
    
}
#[derive(Debug, Clone)]
pub struct TrieController{
    //I want to have a mutex on each vector
    // pub trie: Arc<RwLock<Trie>>,
    //using UUID's converted to string as keys, and Tries as the values (1 trie mapped to each uuid)
    pub trie_map: Arc<RwLock<HashMap<String, Arc<RwLock<Trie>>>>>
}

//have ModelController control synchronization, start with a single trie controller by RwLock
impl TrieController {
    pub fn new(file_path: String) -> Result<Self, CustomError> {
         match Trie::new(file_path){
            (Ok(mut trie), starting_words) => {
                trie.add_words(starting_words);
                let mut trie_map = HashMap::new();
                trie_map.insert(Uuid::new_v4().to_string(), Arc::new(RwLock::new(trie)));
                // println!("trie_map at initialization {:?}", trie_map);
                Ok(TrieController {trie_map: Arc::new(RwLock::new(trie_map))})
            },
            (Err(e), _) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread::{self, sleep};
    use std::time::Duration;

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
        match Trie::new("testing_txt_files/input/invalid_char.txt".to_string()) {
            (Ok(_), _) => unreachable!(),
            //retrieve the invalid 
            (Err(e), _) => { if let CustomError::InvalidCharacter(the_char, the_string) = e {
                assert_eq!(the_char, '9');
                assert_eq!(the_string, "9ichael".to_string());
            } else {
                panic!("Expected InvalidCharacter error, but got {:?}", e);
            }},
        }
    }

    #[test]
    fn new_line() {
        match Trie::new("testing_txt_files/input/newline.txt".to_string()) {
            (Ok(_), _) => assert!(true),//should be no error
            (Err(_), _) => unreachable!(),
        }
        match Trie::new("testing_txt_files/input/newline_times_two.txt".to_string()) {
            (Ok(_), _) => unreachable!(),
            (Err(e), _) => {
                if let CustomError::InvalidFormatting = e {
                    assert!(true);//this means I got error when there are two new lines of expected type
                } 
                else {
                    panic!("Expected InvalidFormatting error, but got {:?}", e);
                }
            },
        }
    }

    #[test]
    fn blank_line() {
        match Trie::new("testing_txt_files/input/blank_line.txt".to_string()) {
            (Ok(_), _) => unreachable!(), 
            (Err(e), _) => {if let CustomError::InvalidFormatting = e {
                assert!(true);//this means I got correct type of error
            } 
            else {
                panic!("Expected InvalidFormatting error, but got {:?}", e);
            }},
        }
    }

    #[test]
    fn create_trie_from_noexistent_file() {
        match Trie::new("doesNOTexist.txt".to_string()) {
            (Ok(_), _) => {
                panic!();
            }, (Err(e), _) => {if let CustomError::UnableToOpen = e {
                assert!(true);//means I was able to get here
            } else {
                panic!("Expected UnableToOpen error, but got {:?}", e);
            }},
        }
    }

    #[test]
    fn retrieve_all_words() {
        match Trie::new("testing_txt_files/input/test2.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);

                let expected_words: HashSet<_> = ["replace", "redfin", "ready", "reps", "brie", "bread", "breed"].map(|x| x.to_string()).iter().cloned().collect();
                let actual_words: HashSet<_> = my_trie.entire_dictionary().iter().cloned().collect();
                assert_eq!(actual_words, expected_words);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn delete_all_words() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                my_trie.delete_dictionary();
                assert_eq!(my_trie.trie_size, 0);
                assert_eq!(my_trie.num_words, 0);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn delete_invalid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.delete_word("salty man".to_string()) {
                    Ok(_) => unreachable!(),
                    Err(e) => {
                        if let CustomError::InvalidCharacter(c, s) = e {
                            assert_eq!(' ', c);
                            assert_eq!(s, "salty man".to_string());
                        } else{
                            unreachable!();
                        }
                    }
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn delete_single_word_should_reduce_size() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                my_trie.delete_word("pepper".to_string());
                assert_eq!(my_trie.trie_size, 23);
                assert_eq!(my_trie.num_words, 6);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn delete_multiple_words_should_not_reduce_size() {
        match Trie::new("testing_txt_files/input/test3.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 20);
                assert_eq!(my_trie.num_words, 7);
                my_trie.delete_word("apple".to_string());
                my_trie.delete_word("applebot".to_string());
                my_trie.delete_word("app".to_string());
                my_trie.delete_word("ap".to_string());
                assert_eq!(my_trie.trie_size, 20);
                assert_eq!(my_trie.num_words, 3);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn add_invalid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.add_words(vec!["saltyman".to_string(), "appstor+E".to_string()]) {
                    Ok(_) => unreachable!(),
                    Err(e) => {
                        if let CustomError::InvalidCharacter(c, s) = e {
                            assert_eq!('+', c);
                            assert_eq!(s, "appstor+E".to_string());
                        } else{
                            unreachable!();
                        }
                    }
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn add_invalid_word_w_spaces() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.add_words(vec!["peppery   giRL".to_string()]) {
                    Ok(_) => unreachable!(),
                    Err(e) => {
                        if let CustomError::InvalidCharacter(c, s) = e {
                            assert_eq!(' ', c);
                            assert_eq!(s, "peppery   giRL".to_string());
                        } else{
                            unreachable!();
                        }
                    }
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn add_uppercase_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
                my_trie.add_words(vec!["BORINGS".to_string()]);
                assert_eq!(my_trie.trie_size, 30);
                assert_eq!(my_trie.num_words, 8);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn prefix_search_invalid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.does_prefix_exist("pepper is my cat".to_string()) {
                    Ok(_) => unreachable!(),
                    Err(e) => {
                        if let CustomError::InvalidCharacter(c, s) = e {
                            assert_eq!(' ', c);
                            assert_eq!(s, "pepper is my cat".to_string());
                        } else{
                            unreachable!();
                        }
                    }
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn prefix_search_valid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.does_prefix_exist("borin".to_string()) {
                    Ok(bool) => assert_eq!(bool, true),
                    Err(_) => unreachable!(),
                }match my_trie.does_prefix_exist("boring".to_string()) {
                    Ok(bool) => assert_eq!(bool, false),
                    Err(_) => unreachable!(),
                }
                match my_trie.does_prefix_exist("borings".to_string()) {
                    Ok(bool) => assert_eq!(bool, false),
                    Err(_) => unreachable!(),
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn prefix_search_uppercase_valid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.does_prefix_exist("appL".to_string()) {
                    Ok(_) => assert!(true),
                    Err(_) => unreachable!(),
                }
                assert_eq!(my_trie.trie_size, 29);
                assert_eq!(my_trie.num_words, 7);
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn word_search_invalid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.does_word_exist("AP7S".to_string()) {
                    Ok(_) => unreachable!(),
                    Err(e) => {
                        if let CustomError::InvalidCharacter(c, s) = e {
                            assert_eq!('7', c);
                            assert_eq!(s, "AP7S".to_string());
                        } else{
                            unreachable!();
                        }
                    }
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn word_search_nonexistent_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.does_word_exist("APpS".to_string()) {
                    Ok(bool) => assert_eq!(false, bool),
                    Err(_) => unreachable!(),
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    #[test]
    fn word_search_uppercase_valid_word() {
        match Trie::new("testing_txt_files/input/test1.txt".to_string()) {
            (Ok(mut my_trie), starting_words) => {
                my_trie.add_words(starting_words);
                match my_trie.does_word_exist("APPlE".to_string()) {
                    Ok(_) => assert!(true),
                    Err(_) => unreachable!(),
                }
            }, (Err(e), _) => panic!("{:?}", e),
        }
    }

    ////END TEST FUNCTIONALITY OF TRIE

    //TEST FUNCTIONALITY OF TRIECONTROLLER
    #[test]
    fn trie_controller_size_zero_no_input_file() {
        match TrieController::new("".to_string()) {
            Ok(trie_controller) => {
                let read_map = trie_controller.trie_map.read().unwrap();
                let key = read_map.iter().next().unwrap().0;
                // println!("Key: {}", key);
                let ref_to_data = read_map.get(key).unwrap().read().unwrap();
                assert_eq!(ref_to_data.trie_size, 0);
                assert_eq!(ref_to_data.num_words, 0);
            }, Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn trie_controller_has_input_file() {
        match TrieController::new("testing_txt_files/input/test1.txt".to_string()) {
            Ok(trie_controller) => {
                let read_map = trie_controller.trie_map.read().unwrap();
                let key = read_map.iter().next().unwrap().0;
                // println!("Key: {}", key);
                let ref_to_data = read_map.get(key).unwrap().read().unwrap();
                assert_eq!(ref_to_data.trie_size, 29);
                assert_eq!(ref_to_data.num_words, 7);
            }, Err(e)  => panic!("{:?}", e),
        }
    }

    #[test]
    fn multithread_read_ops() {
        match TrieController::new("testing_txt_files/input/test2.txt".to_string()) {
            Ok(trie_controller) => {
                let mut handles: Vec<thread::JoinHandle<()>> = vec![];
                let expected_words: HashSet<_> = ["replace", "redfin", "ready", "reps", "brie", "bread", "breed"].map(|x| x.to_string()).iter().cloned().collect();
                let read_map = trie_controller.trie_map.read().unwrap();
                let key = read_map.iter().next().unwrap().0;
                for _ in 0..10 {
                    let my_ref = Arc::clone(&read_map.get(key).unwrap());
                    let copy_expected_words = expected_words.clone();
                    let handle = thread::spawn(move|| {
                        let my_ref = my_ref.read().unwrap();
                        let actual_words: HashSet<_> = my_ref.entire_dictionary().iter().cloned().collect();
                        assert_eq!(actual_words, copy_expected_words);
                        // println!("{:?}", my_ref);
                    });
                    handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            }, Err(e)  => panic!("{:?}", e),
        }
    }
    
    //this test is at the crux of entire design
    #[test]
    fn multithread_rw_ops() {
        match TrieController::new("testing_txt_files/input/test2.txt".to_string()) {
            Ok(trie_controller) => {
                let mut handles: Vec<thread::JoinHandle<()>> = vec![];
                let expected_words: HashSet<_> = ["replace", "redfin", "ready", "reps", "brie", "bread", "breed"].map(|x| x.to_string()).iter().cloned().collect();
                let read_map = trie_controller.trie_map.read().unwrap();
                let key = read_map.iter().next().unwrap().0;
                for i in 0..10 {
                    let my_ref = Arc::clone(&read_map.get(key).unwrap());
                    let copy_expected_words = expected_words.clone();
                    let handle = thread::spawn(move|| {
                        if i == 3 || i == 7 {
                            sleep(Duration::from_secs(i + 1));
                            //make exclusive call to modify trie at different times
                            let mut my_ref = my_ref.write().unwrap();
                            my_ref.delete_dictionary();
                            assert_eq!(my_ref.trie_size, 0);
                        } else{
                            let my_ref = my_ref.read().unwrap();
                            //acquire lock before going to sleep
                            sleep(Duration::from_secs(i + 1));
                            let actual_words: HashSet<_> = my_ref.entire_dictionary().iter().cloned().collect();
                            assert_eq!(actual_words, copy_expected_words);
                        }
                    });
                    handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            }, Err(e)  => panic!("{:?}", e),
        }
    }
    //END TEST FUNCTIONALITY OF TRIECONTROLLER
}
