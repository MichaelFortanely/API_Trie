//TODO define tests in here
pub mod trie {
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::collections::HashMap;

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

impl TrieNode {
    fn new(new_char: char) -> Rc<RefCell<Self>>{
        Rc::new(RefCell::new(TrieNode { is_word: false, char_val: new_char, children: HashMap::new() }))
    }

    fn add_word(&mut self, new_word: String) -> u32 {
        let mut word_iter = new_word.chars();
        if let Some(first_char) = word_iter.next() {
            // println!("first_char is {first_char}");
            if !self.children.contains_key(&first_char) {
                println!("first_char {first_char} not in");
                self.children.insert(first_char, TrieNode::new(first_char));
                if let Some(new_node) = self.children.get_mut(&first_char) {
                    return 1 + new_node.borrow_mut().add_word(new_word[1..].to_string());
                } else{
                    unreachable!();
                }
            } else{
                println!("first_char {first_char} already exists");
                if let Some(existing_node) = self.children.get_mut(&first_char) {
                    return existing_node.borrow_mut().add_word(new_word[1..].to_string());
                } else{
                    unreachable!();
                }
                //the first char of my string is already present in one of my children
            }
        } else{
            //if I have reached the end of my iterator then I will declare that TrieNode I have is the end of a word
            if self.is_word == false{
                self.is_word = true;
                println!("Im at the end of the word! char_val is {}", {self.char_val});
            }
            println!("This word already exists{}", {self.char_val});
            return 0;
        }
    }
}
//base of trie is a trieNode with a value of !
#[derive(Debug)]
pub struct Trie {
    pub base_trie_node: TrieNode,
    pub trie_size: u32,
}

impl Trie{
    //tree will take input characters a-zA-Z, but it will be case insensitive for all methods
    //will return an empty trie if string is blank
    //otherwise for each word that is separated by a new line character, it will be added to the trie
    pub fn new(file_path: String) -> (Result<Self, CustomError>, Vec<String>){
        //need to read in characters from file
        let base_trie_node = Trie { base_trie_node: TrieNode { is_word: false, char_val: '!', children: HashMap::new() }, trie_size: 0 };
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

    pub fn add_starting_words(&mut self, starting_words: Vec<String>){
        //this will equal size of tree, but later on when calling for indiviual words
        //the return type of add_word will return interesting information
        let mut num_nodes_added = 0;
        for word in starting_words {
                num_nodes_added += self.base_trie_node.add_word(word);
                println!("num nodes in the tree is {}", num_nodes_added);
            }
        println!("num nodes in the tree is {}", num_nodes_added);
        self.trie_size = num_nodes_added;
    }
}
}
