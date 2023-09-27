use std::fmt::Debug;
use std::collections::{HashSet, HashMap};


#[derive(Debug)]
pub struct KeywordSpan {
    pub keyword: String,
    pub start: usize,
    pub end: usize,
}


#[derive(Debug, Default)]
struct Node {
    clean_word: Option<String>,
    children: HashMap<String, Node>,  // TODO should this be an Option or just an empty HashMap?
}


#[derive(Debug)]
pub struct KeywordProcessor {
    trie: Node,
    len: usize,  // the number of keywords the struct contains (not the number of nodes)
    non_words_boundaries: HashSet<char>,
    case_sensitive: bool,
}

impl KeywordProcessor {
    pub fn new(case_sensitive: bool) -> Self {
        Self {
            trie: Node::default(),
            len: 0,
            non_words_boundaries: HashSet::from_iter("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_".chars()),
            case_sensitive,
        }
    }

    // fn from(arr: &[&str]) -> Self {
    //     Self::from_iter(arr.into_iter())
    // }

    // fn from_iter<'a, I>(iter: I) -> Self
    // where
    //     I: Iterator<Item = &'a str>,
    // {
    //     let mut this = Self::new();
    //     for word in iter {
    //         this.append(word.to_string());
    //     }
    //     this
    // }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn add_keyword(&mut self, word: &str, clean_word: &str) {
        let normalized_word = {
            if !self.case_sensitive {
                word.to_lowercase()
            } else {
                word.to_string()
            }
        };

        let mut trie = &mut self.trie;

        for word in split_text(&normalized_word, &self.non_words_boundaries) {
            trie = trie.children.entry(word).or_default();
        };

        // increment `len` only if the keyword isn't already there
        if trie.clean_word.is_none() {
            trie.clean_word = Some(clean_word.to_string());
            self.len += 1;
        }
    }

    // pub fn add_keyword_from(&mut self, words: &[(&str, &str)]) {
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

        let mut words = split_text(&text, &self.non_words_boundaries);
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

        let mut words = split_text(&text, &self.non_words_boundaries);
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
                    keywords_found.push(KeywordSpan {
                        keyword: last_keyword_found.unwrap().clone(),
                        start: lst_len[last_kw_found_start_idx],
                        end: lst_len[last_kw_found_end_idx],
                    });

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
        for keyword in self.extract_keywords_with_span(&text) {
            string += &text[prev_end..keyword.start];
            string += &keyword.keyword;
            prev_end = keyword.end;
        }
        string
    }
}


fn split_text(text: &str, non_words_boundaries: &HashSet<char>) -> Vec<String> {
    let mut vec = vec![];
    let mut word = String::new();
    for ch in text.chars() {
        if non_words_boundaries.contains(&ch) {
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
