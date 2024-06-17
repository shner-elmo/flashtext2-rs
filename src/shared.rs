use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, PartialEq, Debug)]
struct Node<'a> {
    clean_word: Option<&'a str>, // TODO: make this an enum that can hold a reference
    children: super::HashMap<'a, Node<'a>>,
}

#[derive(Default, PartialEq, Debug)]
pub struct KeywordProcessor<'a> {
    trie: Node<'a>,
    len: usize, // the number of keywords the struct contains (not the number of nodes)
}

impl<'a> KeywordProcessor<'a> {
    pub fn new() -> Self {
        Self::default()
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
    pub fn add_keyword<S: AsRef<str> + ?Sized>(&mut self, word: &'a S) {
        let word = word.as_ref();
        self.add_keyword_with_clean_word(word, word);
    }

    #[inline]
    pub fn add_keyword_with_clean_word<S: AsRef<str> + ?Sized>(
        &mut self,
        word: &'a S,
        clean_word: &'a S, // make this call an `_impl...()` method that takes an option
    ) {
        let mut trie = &mut self.trie;

        for token in word.as_ref().split_word_bounds() {
            trie = trie.children.entry(token).or_default();
        }

        // increment `len` only if the keyword isn't already there
        if trie.clean_word.is_none() {
            self.len += 1;
        }
        // but even if there is already a keyword, the user can still overwrite its `clean_word`
        trie.clean_word = Some(clean_word.as_ref());
    }

    pub fn add_keywords_from_iter<S: AsRef<str> + ?Sized + 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a S>,
    ) {
        for word in iter {
            self.add_keyword(word.as_ref());
        }
    }

    pub fn add_keywords_with_clean_word_from_iter<S: AsRef<str> + ?Sized + 'a>(
        &mut self,
        iter: impl IntoIterator<Item = (&'a S, &'a S)>,
    ) {
        for (word, clean_word) in iter {
            self.add_keyword_with_clean_word(word.as_ref(), clean_word.as_ref());
        }
    }

    // TODO: should reference to self be like this??
    pub fn extract_keywords(&'a self, text: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        KeywordExtractor::new(text, &self.trie).map(|(keyword, _, _)| keyword)
    }

    pub fn extract_keywords_with_span(
        &'a self,
        text: &'a str,
    ) -> impl Iterator<Item = (&'a str, usize, usize)> + 'a {
        KeywordExtractor::new(text, &self.trie)
    }

    pub fn replace_keywords(&self, text: &str) -> String {
        let mut string = String::with_capacity(text.len());
        // the `prev_end` is necessary to adjust the span as we replace the `word` with its
        // `clean_word`. because if their length is not the same, the next `(start, end)` span
        // won't be accurate.
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

struct KeywordExtractor<'a> {
    idx: usize,
    tokens: Vec<(usize, &'a str)>,
    trie: &'a Node<'a>,
}

impl<'a> KeywordExtractor<'a> {
    fn new(text: &'a str, trie: &'a Node) -> Self {
        Self {
            idx: 0,
            // TODO: instead of saving all of them in memory inside a Vector, we should save
            //  N element inside a Deque (N being the number of levels of the trie??)
            tokens: text.split_word_bound_indices().collect(),
            trie,
        }
    }
}

impl<'a> Iterator for KeywordExtractor<'a> {
    // TODO: return a struct or smth instead of a tuple
    type Item = (&'a str, usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.trie;
        // a keyword is essentially a collection/sequence of tokens
        let mut longest_sequence = None;
        // we need to remember the index that we started traversing the trie, to be able to
        // roll back our `idx` if we are following a "false" sequence, and also to know the
        // span of the sequence if we do find a match.
        let mut traversal_start_idx = self.idx;

        while self.idx < self.tokens.len() {
            let (token_start_idx, token) = self.tokens[self.idx];
            self.idx += 1;

            if let Some(child) = node.children.get(token) {
                node = child;
                if let Some(clean_word) = node.clean_word {
                    longest_sequence = Some((
                        clean_word,
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
        // in which case we will return the last keyword found, or just None.
        longest_sequence
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.tokens.len()))
    }
}
