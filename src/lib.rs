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
    struct UnicaseHashMap<'a, V> {
        inner: std::collections::HashMap<UniCase<&'a str>, V, fxhash::FxBuildHasher>,
    }

    impl<'a, V> UnicaseHashMap<'a, V>
    {
        pub fn entry(&mut self, k: &'a str) -> Entry<UniCase<&'a str>, V> {
            // TODO: make sure its not doing the ASCII check
            // TODO: benchmark `into() vs Unicase::unicode()`
            self.inner.entry(UniCase::unicode(k))
        }

        pub fn get(&self, k: &'a str) -> Option<&V> {
            self.inner.get(&UniCase::unicode(k))
        }
    }

    type HashMap<'a, Node> = UnicaseHashMap<'a, Node>;
    mod shared;
    pub use shared::KeywordProcessor;
}

// TODO: add performance benchmarks using criterion
// TODO: monitor memory usage while running tests (crate `memory_statsCopy item path`)
