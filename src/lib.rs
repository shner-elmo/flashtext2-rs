use std::fmt::Debug;
use std::collections::{HashSet, HashMap};


// use a tuple so its easier to unpack when iterating on the matches (and so the conversion is easier with PyO3)
pub type KeywordSpan = (String, usize, usize);


#[derive(PartialEq, Debug, Default)]
struct Node {
    clean_word: Option<String>,
    children: HashMap<String, Node>,  // TODO should this be an Option or just an empty HashMap?
}


#[derive(PartialEq, Debug)]
pub struct KeywordProcessor {
    trie: Node,
    len: usize,  // the number of keywords the struct contains (not the number of nodes)
    non_word_boundaries: HashSet<char>,
    case_sensitive: bool,
}

impl KeywordProcessor {
    pub fn new(case_sensitive: bool) -> Self {
        Self {
            trie: Node::default(),
            len: 0,
            non_word_boundaries: HashSet::from_iter("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_".chars()),
            case_sensitive,
        }
    }

    pub fn with_non_word_boundaries(chars: HashSet<char>, case_sensitive: bool) -> Self {
        Self { 
            trie: Node::default(),
            len: 0,
            non_word_boundaries: chars,
            case_sensitive,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        // or `self.trie.children.is_empty()`
        self.len == 0
    }

    // we want to keep the implementation of the trie private, because it will probably change in the future
    // fn trie(&self) -> &Node {
    //     &self.trie
    // }

    pub fn add_keyword(&mut self, word: &str, clean_word: &str) {
        let normalized_word = {
            if !self.case_sensitive {
                word.to_lowercase()
            } else {
                word.to_string()
            }
        };

        let mut trie = &mut self.trie;

        for word in split_text(&normalized_word, &self.non_word_boundaries) {
            trie = trie.children.entry(word).or_default();
        };

        // increment `len` only if the keyword isn't already there
        if trie.clean_word.is_none() {
            self.len += 1;
        }
        // but even if there is already a keyword, the user can still overwrite its `clean_word`
        trie.clean_word = Some(clean_word.to_string());
    }

    // pub fn add_keywords_from(&mut self, words: &[(&str, &str)]) {
    //     for (word, clean_word) in words {
    //         self.add_keyword(word, clean_word);
    //     }
    // }

    // TODO: make this a lazy-iterator
    pub fn extract_keywords(&self, text: &str) -> Vec<&String> {
        let text = {
            if !self.case_sensitive {
                text.to_lowercase()
            } else {
                text.to_string()
            }
        };

        let mut words = split_text(&text, &self.non_word_boundaries);
        words.push("".to_string());
        let mut keywords_found = vec![];
        let mut node = &self.trie;

        let mut idx = 0;
        let mut n_words_covered = 0;
        let mut word;
        let mut last_keyword_found = None;

        while idx < words.len() {
            word = &words[idx];
            n_words_covered += 1;

            let children = node.children.get(word);
            if children.is_some() {
                node = children.unwrap();
                if node.clean_word.is_some() {
                    last_keyword_found = Some(node.clean_word.as_ref().unwrap());
                }
            } else {
                if last_keyword_found.is_some() {
                    keywords_found.push(last_keyword_found.unwrap());
                    last_keyword_found = None;
                    idx -= 1;
                } else {
                    idx -= n_words_covered - 1;
                }
                node = &self.trie;
                n_words_covered = 0;
            }
            idx += 1;
        }
        keywords_found
    }

    pub fn extract_keywords_with_span(&self, text: &str) -> Vec<KeywordSpan> {
        let text = {
            if !self.case_sensitive {
                text.to_lowercase()
            } else {
                text.to_string()
            }
        };

        let mut words = split_text(&text, &self.non_word_boundaries);
        words.insert(0, "".to_string());
        words.push("".to_string());

        let mut lst_len = Vec::with_capacity(words.len());
        let mut sum = 0;
        for word in &words {
            sum += word.len();
            lst_len.push(sum);
        }

        let mut keywords_found = vec![];
        let mut node = &self.trie;

        let mut idx = 0;
        let mut n_words_covered = 0;
        let mut word;
        let mut last_keyword_found = None;
        let mut last_kw_found_start_idx = 0;  // default value that will always be overwritten;
        let mut last_kw_found_end_idx = 0;  // default value that will always be overwritten;

        while idx < words.len() {
            word = &words[idx];
            n_words_covered += 1;

            let children = node.children.get(word);
            if children.is_some() {
                node = children.unwrap();
                if node.clean_word.is_some() {
                    last_keyword_found = Some(node.clean_word.as_ref().unwrap());
                    last_kw_found_start_idx = idx - n_words_covered;
                    last_kw_found_end_idx = idx;
                }
            } else {
                if last_keyword_found.is_some() {
                    keywords_found.push((
                        last_keyword_found.unwrap().clone(),
                        lst_len[last_kw_found_start_idx],
                        lst_len[last_kw_found_end_idx],
                    ));

                    last_keyword_found = None;
                    idx -= 1;
                } else {
                    idx -= n_words_covered - 1;
                }
                node = &self.trie;
                n_words_covered = 0;
            }
            idx += 1;
        }
        keywords_found
    }

    pub fn replace_keywords(&self, text: &str) -> String {
        let mut string = String::with_capacity(text.len());
        let mut prev_end = 0;
        for (keyword, start, end) in self.extract_keywords_with_span(&text) {
            string += &text[prev_end..start];
            string += &keyword;
            prev_end = end;
        }
        string += &text[prev_end..];
        string
    }
}

impl Default for KeywordProcessor {
    fn default() -> Self {
        Self::new(false)
    }
}

impl From<&[&str]> for KeywordProcessor {
    fn from(slice: &[&str]) -> Self {
        let mut this = Self::new(false);
        for word in slice {
            this.add_keyword(word, word);
        }
        this
    }
}

impl From<&[(&str, &str)]> for KeywordProcessor {
    fn from(slice:&[(&str, &str)]) -> Self {
        let mut this = Self::new(false);
        for (word, clean_word) in slice {
            this.add_keyword(word, clean_word);
        }
        this
    }
}


// TODO: benchmark this function Vs Regex(r'([^a-zA-Z\d])')
fn split_text(text: &str, non_word_boundaries: &HashSet<char>) -> Vec<String> {
    let mut vec = vec![];
    let mut word = String::new();
    for ch in text.chars() {
        if non_word_boundaries.contains(&ch) {
            word.push(ch);
        } else {
            if word != "" {
                vec.push(word.clone());
                word.clear();
            }
            vec.push(ch.to_string());
        }
    }

    // check if there is a word that we haven't added yet
    if word != "" {
        vec.push(word.clone());
    }
    vec
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let kp = KeywordProcessor {
            trie: Node::default(),
            len: 0,
            case_sensitive: false,
            non_word_boundaries: HashSet::from_iter("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_".chars()),
        };
        assert_eq!(kp, KeywordProcessor::default());
    }

    #[test]
    fn test_split_text() {
        let non_word_boundaries = KeywordProcessor::new(false).non_word_boundaries;

        // empty string shouldn't return anything
        assert!(split_text("", &non_word_boundaries).is_empty());

        let cases = [
            ("Hello", vec!["Hello"]),
            ("Hello ", vec!["Hello", " "]),
            ("Hello World", vec!["Hello", " ", "World"]),
            (" Hello World ", vec![" ", "Hello", " ", "World", " "]),
            ("Hannibal was born in 247 BC, death date; unknown.", vec!["Hannibal", " ", "was", " ", "born", " ", "in", " ", "247", " ", "BC", ",",  " ", "death", " ", "date", ";", " ", "unknown", "."]),
            ("!!'fesf'esfes 32!!..", vec!["!", "!", "'", "fesf", "'", "esfes", " ", "32", "!", "!", ".", "."]),
            ("   py  .  ", vec![" ", " ", " ", "py", " ", " ", ".", " ", " "]),
        ];
        for (string, vec) in cases {
            assert_eq!(split_text(string, &non_word_boundaries), vec);
        }
    }

    #[test]
    fn test_len() {
        // start at zero
        assert_eq!(KeywordProcessor::new(true).len, 0);

        //
        let mut kp = KeywordProcessor::new(true);

        kp.add_keyword("hello", "hey");
        assert_eq!(kp.len, 1);

        kp.add_keyword("hey", "hey");
        assert_eq!(kp.len, 2);

        kp.add_keyword("bye", "hey");
        assert_eq!(kp.len, 3);

        // test same word
        let mut kp = KeywordProcessor::new(true);
        kp.add_keyword("hey", "hey");
        assert_eq!(kp.len, 1);

        kp.add_keyword("hey", "hey");
        assert_eq!(kp.len, 1);

        kp.add_keyword("hey", "bye");
        assert_eq!(kp.len, 1);

        // test same word, different casing (sensitive)
        let mut kp = KeywordProcessor::new(true);
        kp.add_keyword("hey", "hey");
        assert_eq!(kp.len, 1);

        kp.add_keyword("HEY", "hey");
        assert_eq!(kp.len, 2);

        // test same word, different casing (insensitive)
        let mut kp = KeywordProcessor::new(false);
        kp.add_keyword("hey", "hey");
        assert_eq!(kp.len, 1);

        kp.add_keyword("HEY", "hey");
        assert_eq!(kp.len, 1);
    }

    #[test]
    fn test_add_keyword() {
        // empty
        let kp = KeywordProcessor::new(true);
        let trie = Node {
            clean_word: None,
            children: HashMap::new(),
        };
        assert_eq!(kp.trie, trie);

        // test few keywords
        let mut kp = KeywordProcessor::new(true);
        kp.add_keyword("hey", "Hey");
        kp.add_keyword("hello", "Hello!");
        kp.add_keyword("hello world", "Hello World");
        kp.add_keyword("C# is no good :(", "C# bad");

        let trie = Node {
            clean_word: None,
            children: HashMap::from([
                (
                    "hey".to_string(),
                    Node { clean_word: Some("Hey".to_string()), children: HashMap::new()},
                ),
                (
                    "hello".to_string(),
                    Node { clean_word: Some("Hello!".to_string()), children: HashMap::from([
                        (
                            " ".to_string(),
                            Node { clean_word: None, children: HashMap::from([
                                (
                                    "world".to_string(),
                                    Node { clean_word: Some("Hello World".to_string()), children: HashMap::new()},
                                ),
                            ])}
                        ),
                    ])},
                ),
                (
                    "C".to_string(),
                    Node { clean_word: None, children: HashMap::from([
                        (
                            "#".to_string(),
                            Node { clean_word: None, children:  HashMap::from([
                                (
                                    " ".to_string(),
                                    Node { clean_word: None, children:  HashMap::from([
                                        (
                                            "is".to_string(),
                                            Node { clean_word: None, children:  HashMap::from([
                                                (
                                                    " ".to_string(),
                                                    Node { clean_word: None, children:  HashMap::from([
                                                        (
                                                            "no".to_string(),
                                                            Node { clean_word: None, children:  HashMap::from([
                                                                (
                                                                    " ".to_string(),
                                                                    Node { clean_word: None, children:  HashMap::from([
                                                                        (
                                                                            "good".to_string(),
                                                                            Node { clean_word: None, children:  HashMap::from([
                                                                                (
                                                                                    " ".to_string(),
                                                                                    Node { clean_word: None, children:  HashMap::from([
                                                                                        (
                                                                                            ":".to_string(),
                                                                                            Node { clean_word: None, children:  HashMap::from([
                                                                                                (
                                                                                                    "(".to_string(),
                                                                                                    Node { clean_word: Some("C# bad".to_string()), children:  HashMap::new() }
                                                                                                )
                                                                                            ])},
                                                                                        ),
                                                                                    ])},
                                                                                ),
                                                                            ])},
                                                                        ),
                                                                    ])},
                                                                ),
                                                            ])},
                                                        ),
                                                    ])},
                                                ),
                                            ])},
                                        ),
                                    ])},
                                ),
                            ])},
                        ),
                    ])},
                ),
            ]),
        };
        assert_eq!(kp.trie, trie);
    }
}

// TODO: move these tests to a separate module (but they need to access private Structs/fields)!!
