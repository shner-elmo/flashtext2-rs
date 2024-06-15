# Flashtext2

This crate allows you to extract & replace strings very efficiently, and with better
performance than using RegEx.

Its especially performant when you have a very big list of keywords that you want to
extract from your text, and also for replacing many values.

## How it works

The [flashtext](https://arxiv.org/abs/1711.00046) algorithm uses a trie to save all the
keywords the user wants to extract, a keyword is defined as a sequence of tokens,
for example `"Hello world!"` becomes: `["Hello", " ", "world", "!"]`.
And in this implementation, each node in the trie contains one token (not character!).  
The tokens are split using the [Unicode Standard Annex #29](https://www.unicode.org/reports/tr29/).

## Time complexity

The time complexity of this algorithm is not related to the number of keywords in the trie,
but only by the length of the document!


## Quick start

```rust
use flashtext2::case_sensitive::KeywordProcessor;

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

The `KeywordProcessor` struct is defined in two modules: `case_sensitive` and `case_insensitive`.
Both modules provide the same methods and signatures; however, the internal string storage
differs. The `case_insensitive` module utilizes a case-insensitive hashmap ([`case_insensitive_hashmap`]).

```rust
use flashtext2::case_insensitive::KeywordProcessor;

let mut kp = KeywordProcessor::new();
kp.add_keywords_from_iter(["Foo", "Bar"]);

let text = "Foo BaR foO FOO";
let keywords: Vec<_> = kp
    .extract_keywords(text)
    .collect();
assert_eq!(keywords, ["Foo", "Bar", "Foo", "Foo"]);
```

The [`unicase`](https://docs.rs/unicase/latest/unicase/) crate accurately processes and matches
keywords despite variations in case and more complex characters:
```rust
use flashtext2::case_insensitive::KeywordProcessor;

let mut kp = KeywordProcessor::new();
let tokens = ["flour", "Maße", "ᾲ στο διάολο"];
kp.add_keywords_from_iter(tokens);

let text = "ﬂour, MASSE, ὰι στο διάολο";
let found_tokens: Vec<_> = kp.extract_keywords(text).collect();
assert_eq!(found_tokens, tokens);
```

[`case_insensitive_hashmap`]: https://docs.rs/case_insensitive_hashmap/latest/case_insensitive_hashmap

