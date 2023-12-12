//! # Flashtext2
//!
//! This crate allows you to extract & replace strings very efficiently, and with better
//! performance than using RegEx.
//!
//! Its especially performant when you have a have a very big list of keywords that you want to
//! extract from your text, and also for replace many values.
//!
//! ## How it works
//!
//! The [flashtext](https://arxiv.org/abs/1711.00046) algorithm uses a trie to save all the
//! keywords the user wants to extract, a keyword is defined a sequence of tokens,
//! for example `"Hello world!"` becomes: `["Hello", " ", "world", "!"]`.
//! And in this implementation, each node in the trie contains one token (not character!).
//! (The tokens are split using the [Unicode Standard Annex #29](https://www.unicode.org/reports/tr29/))
//!
//! ## Time complexity
//!
//! The time complexity of this algorithm is not related to the number of keywords in the trie,
//! but only by the length of the document!
//!
//!
//! ## Quick start
//!
//! ```
//! use flashtext2::KeywordProcessor;
//!
//! fn main() {
//!     let mut kp = KeywordProcessor::new();
//!     kp.add_keyword("love");
//!     kp.add_keyword("Rust");
//!     kp.add_keyword("Hello");
//!
//!     assert_eq!(kp.len(), 3);
//!
//!     // extract keywords
//!     let keywords_found: Vec<_> = kp
//!         .extract_keywords("Hello, I love programming in Rust!")
//!         .collect();
//!     assert_eq!(keywords_found, ["Hello", "love", "Rust"]);
//!
//!     // extract keywords with span
//!     let keywords_with_span: Vec<_> = kp
//!         .extract_keywords_with_span("Hello, I love programming in Rust!")
//!         .collect();
//!     assert_eq!(keywords_with_span, [("Hello", 0, 5), ("love", 9, 13), ("Rust", 29, 33)]);
//!
//!     // replace keywords
//!     let mut kp = KeywordProcessor::new();
//!     kp.add_keyword_with_clean_word("Hello", "Hey");
//!     kp.add_keyword_with_clean_word("love", "hate");
//!     kp.add_keyword_with_clean_word("Rust", "Java");
//!
//!     let replaced_text = kp
//!         .replace_keywords("Hello, I love programming in Rust!");
//!     assert_eq!(replaced_text, "Hey, I hate programming in Java!");
//! }
//!```
//!
//! ## Case insensitive
//!
//! At the moment this crate doesn't support case-insensitive search, although its something I want
//! to add in the future.
//! As a workaround you can normalize the text by calling `str::to_lowercase()` when inserting the
//! keywords and also on the text you want to search, i.e.:
//!
//! ```
//! use flashtext2::KeywordProcessor;
//!
//! let mut kp = KeywordProcessor::new();
//! kp.add_keywords_from_iter(["Rust", "PERFORMANT"].map(str::to_lowercase));
//!
//! let text = &"Rust is great because its very performant".to_lowercase();
//! let keywords: Vec<_> = kp
//!     .extract_keywords(text)
//!     .collect();
//! assert_eq!(keywords, ["rust", "performant"]);
//! ```
//!

use std::fmt::Debug;

use fxhash::FxHashMap as HashMap;
use unicode_segmentation::UnicodeSegmentation;

#[derive(PartialEq, Debug, Default)]
struct Node {
    clean_word: Option<String>, // if clean_word is `Some` it means that we've reached the end of a keyword
    children: HashMap<String, Node>,
}

#[derive(PartialEq, Debug)]
pub struct KeywordProcessor {
    trie: Node,
    len: usize, // the number of keywords the struct contains (not the number of nodes)
}

impl KeywordProcessor {
    pub fn new() -> Self {
        Self {
            trie: Node::default(),
            len: 0,
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

    #[inline]
    pub fn add_keyword<T: AsRef<str> + Clone>(&mut self, word: T) {
        let clean_word = word.clone();
        self.add_keyword_with_clean_word(word, clean_word);
    }

    #[inline]
    pub fn add_keyword_with_clean_word<T: AsRef<str>>(&mut self, word: T, clean_word: T) {
        let mut trie = &mut self.trie;

        for token in word.as_ref().split_word_bounds() {
            trie = trie.children.entry(token.to_string()).or_default();
        }

        // increment `len` only if the keyword isn't already there
        if trie.clean_word.is_none() {
            self.len += 1;
        }
        // but even if there is already a keyword, the user can still overwrite its `clean_word`
        trie.clean_word = Some(clean_word.as_ref().to_string());
    }

    pub fn add_keywords_from_iter<I, T>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str> + Clone,
    {
        for word in iter {
            self.add_keyword(word);
        }
    }

    pub fn add_keywords_with_clean_word_from_iter<I, T>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (T, T)>,
        T: AsRef<str> + Clone,
    {
        for (word, clean_word) in iter {
            self.add_keyword_with_clean_word(word, clean_word);
        }
    }

    pub fn extract_keywords<'a>(&'a self, text: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        KeywordExtractor::new(text, &self.trie).map(|(keyword, _, _)| keyword)
    }

    pub fn extract_keywords_with_span<'a>(
        &'a self,
        text: &'a str,
    ) -> impl Iterator<Item = (&'a str, usize, usize)> + 'a {
        KeywordExtractor::new(text, &self.trie)
    }

    pub fn replace_keywords(&self, text: &str) -> String {
        let mut string = String::with_capacity(text.len());
        // the `prev_end` is necessary to adjust the span as we replace the `word` with its
        // `clean_word`. because if their length is not the same, the next `(start, end)` span
        // wont be accurate.
        let mut prev_end = 0;
        for (keyword, start, end) in self.extract_keywords_with_span(text) {
            string += &text[prev_end..start];
            string += &keyword;
            prev_end = end;
        }
        string += &text[prev_end..];

        // if a `word` is bigger than its `clean_word` then it will over-allocate
        string.shrink_to_fit();

        string
    }
}

impl Default for KeywordProcessor {
    fn default() -> Self {
        Self::new()
    }
}

struct KeywordExtractor<'a> {
    idx: usize,
    tokens: Vec<(usize, &'a str)>,
    trie: &'a Node,
}

impl<'a> KeywordExtractor<'a> {
    fn new(text: &'a str, trie: &'a Node) -> Self {
        Self {
            idx: 0,
            tokens: text.split_word_bound_indices().collect(),
            trie,
        }
    }
}

impl<'a> Iterator for KeywordExtractor<'a> {
    type Item = (&'a str, usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.trie;
        // a keyword is essentially a collection/sequence of tokens
        let mut longest_sequence = None;
        // we need to remember the index that we started traversing the trie, to be able to
        // rollback our `idx` if we are following a "fake" sequence, and also to know the
        // span of the sequence if we do find a match.
        let mut traversal_start_idx = self.idx;

        while self.idx < self.tokens.len() {
            let (token_start_idx, token) = self.tokens[self.idx];
            self.idx += 1;

            if let Some(child) = node.children.get(token) {
                node = child;
                if let Some(clean_word) = &node.clean_word {
                    longest_sequence = Some((
                        clean_word.as_str(),
                        self.tokens[traversal_start_idx].0,
                        token_start_idx + token.len(),
                    ));
                }
            } else {
                if let kw @ Some(_) = longest_sequence {
                    self.idx -= 1;
                    return kw;
                } else {
                    self.idx = traversal_start_idx + 1;
                    // reset the state as above
                    node = self.trie;
                    traversal_start_idx = self.idx;
                }
            }
        }

        // we will reach this code only in the last item of the iterator,
        // in which case we will return the longest found keyword, or just None.
        longest_sequence
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.tokens.len()))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::HashSet;
//
//     #[test]
//     fn test_default() {
//         let kp = KeywordProcessor {
//             trie: Node::default(),
//             len: 0,
//         };
//         assert_eq!(kp, KeywordProcessor::default());
//     }
//
//     // #[test]
//     // fn from_iter_strings() {
//     //     let kp = KeywordProcessor::from_iter(["hello", "world"]);
//     //     assert_eq!(kp.extract_keywords("hello world").collect::<Vec<_>>(), ["hello", "world"]);
//     //
//     //     let kp = KeywordProcessor::from_iter(vec!["hello", "world"]);
//     //     assert_eq!(kp.extract_keywords("hello world").collect::<Vec<_>>(), ["hello", "world"]);
//     //
//     //     let kp = KeywordProcessor::from_iter(HashSet::from(["hello", "world"]));
//     //     assert_eq!(kp.extract_keywords("hello world").collect::<Vec<_>>(), ["hello", "world"]);
//     //
//     //     let kp = KeywordProcessor::from_iter([&"hello", &"world"]);
//     //     assert_eq!(kp.extract_keywords("hello world").collect::<Vec<_>>(), ["hello", "world"]);
//     //
//     //     let kp = KeywordProcessor::from_iter(["hello".to_string(), "world".to_string()]);
//     //     assert_eq!(kp.extract_keywords("hello world").collect::<Vec<_>>(), ["hello", "world"]);
//     //
//     //     let kp = KeywordProcessor::from_iter([&"hello".to_string(), &"world".to_string()]);
//     //     assert_eq!(kp.extract_keywords("hello world").collect::<Vec<_>>(), ["hello", "world"]);
//     // }
//
//     #[test]
//     fn test_split_text() {
//         // empty string shouldn't return anything
//         assert!("".split_word_bounds().next().is_none());
//
//         assert_eq!(" ".split_word_bounds().collect::<Vec<_>>(), vec![" "]);
//
//         let cases = [
//             ("Hello", vec!["Hello"]),
//             ("Hello ", vec!["Hello", " "]),
//             ("Hello World", vec!["Hello", " ", "World"]),
//             (" Hello World ", vec![" ", "Hello", " ", "World", " "]),
//             (
//                 "Hannibal was born in 247 BC, death date; unknown.",
//                 vec![
//                     "Hannibal", " ", "was", " ", "born", " ", "in", " ", "247", " ", "BC", ",",
//                     " ", "death", " ", "date", ";", " ", "unknown", ".",
//                 ],
//             ),
//             (
//                 "!!'fesf'esfes 32!!..",
//                 vec!["!", "!", "'", "fesf'esfes", " ", "32", "!", "!", ".", "."],
//             ),
//             ("   py  .  ", vec!["   ", "py", "  ", ".", "  "]),
//         ];
//         for (string, vec) in cases {
//             assert_eq!(string.split_word_bounds().collect::<Vec<_>>(), vec);
//         }
//     }
//
//     #[test]
//     fn test_len() {
//         // start at zero
//         assert_eq!(KeywordProcessor::new().len, 0);
//
//         //
//         let mut kp = KeywordProcessor::new();
//
//         kp.add_keyword("hello", "hey");
//         assert_eq!(kp.len, 1);
//
//         kp.add_keyword("hey", "hey");
//         assert_eq!(kp.len, 2);
//
//         kp.add_keyword("bye", "hey");
//         assert_eq!(kp.len, 3);
//
//         // test same word
//         let mut kp = KeywordProcessor::new();
//         kp.add_keyword("hey", "hey");
//         assert_eq!(kp.len, 1);
//
//         kp.add_keyword("hey", "hey");
//         assert_eq!(kp.len, 1);
//
//         kp.add_keyword("hey", "bye");
//         assert_eq!(kp.len, 1);
//
//         // test same word, different casing (sensitive)
//         let mut kp = KeywordProcessor::new();
//         kp.add_keyword("hey", "hey");
//         assert_eq!(kp.len, 1);
//
//         kp.add_keyword("HEY", "hey");
//         assert_eq!(kp.len, 2);
//
//         // test same word, different casing (insensitive)
//         let mut kp = KeywordProcessor::new();
//         kp.add_keyword("hey", "hey");
//         assert_eq!(kp.len, 1);
//
//         kp.add_keyword("HEY", "hey");
//         assert_eq!(kp.len, 1);
//     }
//
//     #[test]
//     fn extractor() {
//         // keywords, text, output
//         let arr = [
//             (1, vec!["hello"], "hello", vec!["hello"]),
//             (2, vec!["hello"], " hello", vec!["hello"]),
//             (3, vec!["hello"], "hello ", vec!["hello"]),
//             (4, vec!["hello"], " hello ", vec!["hello"]),
//             (5, vec!["hello"], "  hello  ", vec!["hello"]),
//             (6, vec!["  hello  "], "  hello  ", vec!["  hello  "]),
//             (7, vec!["hello world"], "hello world", vec!["hello world"]), // multi word
//             (
//                 8,
//                 vec!["hello world"],
//                 "hello hello world",
//                 vec!["hello world"],
//             ),
//             (
//                 9,
//                 vec!["hello", "world", "hello world"],
//                 "hello hello world",
//                 vec!["hello", "hello world"],
//             ),
//             (
//                 10,
//                 vec!["hello", "world", " hello world"],
//                 "hello hello world",
//                 vec!["hello", " hello world"],
//             ),
//             (
//                 11,
//                 vec!["hello", "world", "  hello world"],
//                 "hello  hello world",
//                 vec!["hello", "  hello world"],
//             ),
//             // TODO make this cases work (whitespace should be split in strings of len == 1)
//             // (
//             //     12,
//             //     vec!["hello", "world", " hello world"],
//             //     "hello  hello world",
//             //     vec!["hello", " hello world"],
//             // ),
//             // (
//             //     13,
//             //     vec!["hello", "world", "   hello world"],
//             //     "hello    hello world",
//             //     vec!["hello", "   hello world"],
//             // ),
//         ];
//         for (test_id, keywords, text, output) in arr {
//             println!(
//                 "test ID: {} --------------------------------------------------------------",
//                 test_id
//             );
//
//             assert!(HashSet::<&&str>::from_iter(&output).is_subset(&HashSet::from_iter(&keywords)));
//
//             let mut kp = KeywordProcessor::new();
//             for kw in keywords {
//                 kp.add_keyword(kw, kw);
//             }
//             let vec: Vec<_> = kp.extract_keywords(text).collect();
//             assert_eq!(vec, output, "Test case: {}", test_id);
//         }
//     }
//
//     // #[test]
//     // fn test_add_keyword() {
//     //     // empty
//     //     let kp = KeywordProcessor::new(true);
//     //     let trie = Node {
//     //         clean_word: None,
//     //         children: HashMap::default(),
//     //     };
//     //     assert_eq!(kp.trie, trie);
//     //
//     //     // test few keywords
//     //     let mut kp = KeywordProcessor::new(true);
//     //     kp.add_keyword("hey", "Hey");
//     //     kp.add_keyword("hello", "Hello!");
//     //     kp.add_keyword("hello world", "Hello World");
//     //     kp.add_keyword("C# is no good :(", "C# bad");
//     //
//     //     let trie = Node {
//     //         clean_word: None,
//     //         children: HashMap::from([
//     //             (
//     //                 "hey".to_string(),
//     //                 Node { clean_word: Some("Hey".to_string()), children: HashMap::default()},
//     //             ),
//     //             (
//     //                 "hello".to_string(),
//     //                 Node { clean_word: Some("Hello!".to_string()), children: HashMap::from([
//     //                     (
//     //                         " ".to_string(),
//     //                         Node { clean_word: None, children: HashMap::from([
//     //                             (
//     //                                 "world".to_string(),
//     //                                 Node { clean_word: Some("Hello World".to_string()), children: HashMap::default()},
//     //                             ),
//     //                         ])}
//     //                     ),
//     //                 ])},
//     //             ),
//     //             (
//     //                 "C".to_string(),
//     //                 Node { clean_word: None, children: HashMap::from([
//     //                     (
//     //                         "#".to_string(),
//     //                         Node { clean_word: None, children:  HashMap::from([
//     //                             (
//     //                                 " ".to_string(),
//     //                                 Node { clean_word: None, children:  HashMap::from([
//     //                                     (
//     //                                         "is".to_string(),
//     //                                         Node { clean_word: None, children:  HashMap::from([
//     //                                             (
//     //                                                 " ".to_string(),
//     //                                                 Node { clean_word: None, children:  HashMap::from([
//     //                                                     (
//     //                                                         "no".to_string(),
//     //                                                         Node { clean_word: None, children:  HashMap::from([
//     //                                                             (
//     //                                                                 " ".to_string(),
//     //                                                                 Node { clean_word: None, children:  HashMap::from([
//     //                                                                     (
//     //                                                                         "good".to_string(),
//     //                                                                         Node { clean_word: None, children:  HashMap::from([
//     //                                                                             (
//     //                                                                                 " ".to_string(),
//     //                                                                                 Node { clean_word: None, children:  HashMap::from([
//     //                                                                                     (
//     //                                                                                         ":".to_string(),
//     //                                                                                         Node { clean_word: None, children:  HashMap::from([
//     //                                                                                             (
//     //                                                                                                 "(".to_string(),
//     //                                                                                                 Node { clean_word: Some("C# bad".to_string()), children:  HashMap::default() }
//     //                                                                                             )
//     //                                                                                         ])},
//     //                                                                                     ),
//     //                                                                                 ])},
//     //                                                                             ),
//     //                                                                         ])},
//     //                                                                     ),
//     //                                                                 ])},
//     //                                                             ),
//     //                                                         ])},
//     //                                                     ),
//     //                                                 ])},
//     //                                             ),
//     //                                         ])},
//     //                                     ),
//     //                                 ])},
//     //                             ),
//     //                         ])},
//     //                     ),
//     //                 ])},
//     //             ),
//     //         ]),
//     //     };
//     //     assert_eq!(kp.trie, trie);
//     // }
// }
//
// // TODO: move these tests to a separate module (but they need to access private Structs/fields)!!
