#![doc = include_str!("../README.md")]

#[path = "."]
pub mod case_sensitive {
    type HashMap<'a, Node> = std::collections::HashMap<&'a str, Node, fxhash::FxBuildHasher>;
    mod shared;
    pub use shared::KeywordProcessor;
}

#[path = "."]
pub mod case_insensitive {
    use std::collections::hash_map::Entry;
    use unicase::UniCase;

    #[derive(Debug, Default, PartialEq)]
    struct UnicaseHashMap<K: AsRef<str>, V> {
        inner: std::collections::HashMap<UniCase<K>, V, fxhash::FxBuildHasher>,
    }

    impl<K, V> UnicaseHashMap<K, V>
    where
        K: AsRef<str>,
    {
        pub fn entry<Q: Into<UniCase<K>>>(&mut self, k: Q) -> Entry<UniCase<K>, V> {
            // TODO: make sure its not doing the ASCII check
            // TODO: benchmark `into() vs Unicase::unicode()`
            self.inner.entry(k.into())
        }

        pub fn get<Q: Into<UniCase<K>>>(&self, k: Q) -> Option<&V> {
            self.inner.get(&k.into())
        }
    }

    type HashMap<'a, Node> = UnicaseHashMap<&'a str, Node>;
    mod shared;
    pub use shared::KeywordProcessor;
}

// TODO: add performance benchmarks using criterion
// TODO: monitor memory usage while running tests (crate `memory_statsCopy item path`)
