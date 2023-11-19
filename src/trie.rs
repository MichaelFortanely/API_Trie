//TODO define tests in here
pub mod trie {
use std::iter::Peekable;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::collections::HashMap;
use std::str::Chars;

#[derive(Debug)]
pub enum CustomError {
    InvalidFormatting,
    InvalidCharacter(char),
    UnableToOpen(std::io::Error)
}

//I should change my implementation to use map of char to Rc<RefCell<TrieNode>>
//to 
#[derive(Debug)]
pub struct TrieNode {
    is_word: bool,
    char_val: char,
    children: HashMap<char, Rc<RefCell<TrieNode>>>,
}
//having helper functions that return refernces can get really complicated
//try to implement this without many helper functions i.e. references that return TrieNodes bc then stuff gets really complicated


impl TrieNode {
    fn new(new_char: char) -> Rc<RefCell<Self>>{
        Rc::new(RefCell::new(TrieNode { is_word: false, char_val: new_char, children: HashMap::new() }))
    }

    fn _search_tree(&self, mut chars: Peekable<Chars>, must_be_complete: bool, suffic_vec: &mut Vec<String>, get_suffixes: bool) -> bool{
        match chars.next() {
            Some(_) => {
                // println!("char is {curr_char} and self.char_val is {}", self.char_val);
                    if let Some(&next_char) = chars.peek() {
                        match self.children.get(&next_char) {
                            Some(value_from_key) => {
                                //next trie node exists
                                return value_from_key.borrow()._search_tree(chars, must_be_complete, suffic_vec, get_suffixes)
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
                                self._autocomplete(&mut mut_string, suffic_vec, 3);
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

    fn _autocomplete(&self, s: &mut String, suffix_vec: &mut Vec<String>, c: u32) {
        //do a DFS from this point  -> add current nodes 
        //I will only need to do the cloning when I am at a word (leaf or node says is word)
        //use this as template when making tests later
        // println!("in _auto_complete");
        // if c == 3{
        //     s.push('a');
        //     suffix_vec.push(s.clone());
        // }
        // if c == 2{
        //     s.push('b');
        // }
        // if c == 1{
        //     s.push('c');
        //     suffix_vec.push(s.clone());
        // }
        // if c == 0{
        //     s.push('d');
        //     return;
        // }
        // self._autocomplete(s, suffix_vec, c - 1);
        //
    }

    //u32 is number of new nodes inserted, bool is whether this is a new word
    fn _add_word(&mut self, mut new_word: impl Iterator<Item = char>) -> (u32, bool) {
        if let Some(first_char) = new_word.next() {
            let mut val_to_add = 0;
            if !self.children.contains_key(&first_char) {
                println!("{first_char} not in nodes children");
                self.children.insert(first_char, TrieNode::new(first_char));
                val_to_add = 1;
            } else{
                println!("{first_char} exists in node children");
            }
            let returned_val = self.children.get_mut(&first_char).unwrap().borrow_mut()._add_word(new_word);
            return (val_to_add + returned_val.0, returned_val.1);
        } else{
            //if I have reached the end of my iterator then I will declare that TrieNode I have is the end of a word
            if self.is_word == false{
                self.is_word = true;
                println!("Im at the end of a new word! char_val is {}", {self.char_val});
                return (0, true)
            }
            println!("This word already exists{}", {self.char_val});
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
        //this will equal size of tree, but later on when calling for indiviual words
        //the return type of add_word will return interesting information
        let mut num_nodes_added = 0;
        for word in starting_words {
                let returned_tup = self.base_trie_node._add_word(word.chars());
                num_nodes_added += returned_tup.0;
                if returned_tup.1 == true {
                    println!("is a new word");
                    self.num_words += 1;
                } else{
                    println!("not a new word");
                }
                println!("num nodes in the tree is {}", num_nodes_added);
            }
        println!("num nodes in the tree is {}", num_nodes_added);
        self.trie_size = num_nodes_added;
    }
    //NOTE -> base of tree is TrieNode with value !
    //this function returns true only if the string is 
    pub fn does_prefix_exist(&self, s: String) -> bool {
        let mut placeholder: Vec<String> = vec![];
        self.base_trie_node._search_tree(("!".to_string() + &(s.to_ascii_lowercase())).chars().peekable(), false, &mut placeholder, false)
    }

    //this function returns true if the word has been added to the trie
    pub fn does_word_exist(&self, s: String) -> bool {
        let mut placeholder: Vec<String> = vec![];
        self.base_trie_node._search_tree(("!".to_string() + &(s.to_ascii_lowercase())).chars().peekable(), true, &mut placeholder, false)
    }

    //the two functions above have placholder vectors
    //I will do a DFS using a string that will be added to the vector that is passed in as a mutable reference so it does not need to be returned

    //will return with blank if not even a prefix does not work
    pub fn autocomplete(&self, s: String) -> Vec<String>{
        let mut suffix_list: Vec<String> = vec![];
        self.base_trie_node._search_tree(("!".to_string() + &(s.to_ascii_lowercase())).chars().peekable(), false, &mut suffix_list, true);
        suffix_list
    }
}
}

//TODO add tests
