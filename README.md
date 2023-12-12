# flashtext2-rs
Flashtext implementation in Rust

# Flashtext2

This crate allows you to extract & replace strings very efficiently, and with better
performance than using RegEx.

Its especially performant when you have a have a very big list of keywords that you want to
extract from your text, and also for replace many values.

## How it works

The [flashtext](https://arxiv.org/abs/1711.00046) algorithm uses a trie to save all the
keywords the user wants to extract, a keyword is defined a sequence of tokens,
for example `"Hello world!"` becomes: `["Hello", " ", "world", "!"]` (the tokens are split using the [Unicode Standard Annex #29](https://www.unicode.org/reports/tr29/)).

And in this implementation, each node in the trie contains one token (not character!).

## Time complexity

The time complexity of this algorithm is not related to the number of keywords in the trie,
but only by the length of the document!


## Quick start

```rust
use flashtext2::KeywordProcessor;

fn main() {
    let mut kp = KeywordProcessor::new();
    kp.add_keyword("love");
    kp.add_keyword("Rust");
    kp.add_keyword("Hello");
    
    assert_eq!(kp.len(), 3);
    
    // extract keywords
    let keywords_found: Vec<_> = kp
        .extract_keywords("Hello, I love programming in Rust!")
        .collect();
    assert_eq!(keywords_found, ["Hello", "love", "Rust"]);
    
    // extract keywords with span
    let keywords_with_span: Vec<_> = kp
        .extract_keywords_with_span("Hello, I love programming in Rust!")
        .collect();
    assert_eq!(keywords_with_span, [("Hello", 0, 5), ("love", 9, 13), ("Rust", 29, 33)]);
    
    // replace keywords
    let mut kp = KeywordProcessor::new();
    kp.add_keyword_with_clean_word("Hello", "Hey");
    kp.add_keyword_with_clean_word("love", "hate");
    kp.add_keyword_with_clean_word("Rust", "Java");
    
    let replaced_text = kp
        .replace_keywords("Hello, I love programming in Rust!");
    assert_eq!(replaced_text, "Hey, I hate programming in Java!");
}
```

## Case insensitive

At the moment this crate doesn't support case-insensitive search, although its something I want
to add in the future.
As a workaround you can normalize the text by calling `str::to_lowercase()` when inserting the
keywords and also on the text you want to search, i.e.:

```rust
use flashtext2::KeywordProcessor;

fn main() {
    let mut kp = KeywordProcessor::new();
    kp.add_keywords_from_iter(["Rust", "PERFORMANT"].map(str::to_lowercase));

    let text = &"Rust is great because its very performant".to_lowercase();
    let keywords: Vec<_> = kp
        .extract_keywords(text)
        .collect();
    assert_eq!(keywords, ["rust", "performant"]);
}
```
