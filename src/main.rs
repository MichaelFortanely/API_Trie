use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::collections::HashMap;
//define an enum that has custom error types
fn main() {
    println!("Hello, world!");
    let first = TrieNode::new('a');
    println!("{:?}",first);
    match Trie::new(String::from("s.txt")){
        (Ok(mut my_trie), starting_words) => {
            for word in starting_words {
                my_trie.base_trie_node.add_word(word);
            }
            println!("success")
        },
        (Err(e), _) => print!("had error {:?}", e),
    }
}
//1. taking in new words to add to the trie, 
//2. giving auto complete suggestions, and 
//3. giving a bool for if the word exists in the trie or not
//4. size of the Trie or the number of nodes
//how to implement a trie --> do this first in sync manner and then consider async
#[derive(Debug)]
enum CustomError {
    InvalidFormatting,
    InvalidCharacter(char),
    UnableToOpen(std::io::Error)
}

//I should change my implementation to use map of char to Rc<RefCell<TrieNode>>
//to 
#[derive(Debug)]
struct TrieNode {
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
                self.children.insert(first_char, Rc::new(RefCell::new(TrieNode { is_word: false, char_val: first_char, children: HashMap::new() })));
                if let Some(new_node) = self.children.get_mut(&first_char) {

                    new_node.borrow_mut().add_word(new_word[1..].to_string());
                } else{
                    unreachable!();
                }
            } else{
                println!("first_char {first_char} already exists");
                if let Some(existing_node) = self.children.get_mut(&first_char) {
                    existing_node.borrow_mut().add_word(new_word[1..].to_string());
                } else{
                    unreachable!();
                }
                //the first char of my string is already present in one of my children

            }
        } else{
            //if I have reached the end of my iterator then I will declare that TrieNode I have is the end of a word
            self.is_word = true;
            println!("Im at the end of the word! char_val is {}", {self.char_val})
        }

        1
    }
}
//base of trie is a trieNode with a value of !
#[derive(Debug)]
struct Trie {
    base_trie_node: TrieNode,
    trie_size: u32,
}

impl Trie{
    //tree will take input characters a-zA-Z, but it will be case insensitive for all methods
    //will return an empty trie if string is blank
    //otherwise for each word that is separated by a new line character, it will be added to the trie
    fn new(file_path: String) -> (Result<Self, CustomError>, Vec<String>){
        //need to read in characters from file
        if file_path.len() == 0 {
            return (Ok(Trie { base_trie_node: TrieNode { is_word: false, char_val: '!', children: HashMap::new() }, trie_size: 0 }), vec![])
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
        (Ok(Trie { base_trie_node: TrieNode { is_word: false, char_val: '!', children: HashMap::new() }, trie_size: 0 }), verified_contents)
    }
}
