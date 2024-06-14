#[path = "."]
pub mod case_sensitive {
    type HashMap<Node> = std::collections::HashMap<String, Node, fxhash::FxBuildHasher>;
    mod shared;
    pub use shared::{KeywordProcessor, Node};
}

#[path = "."]
pub mod case_insensitive {
    type HashMap<Node> =
        case_insensitive_hashmap::CaseInsensitiveHashMap<Node, fxhash::FxBuildHasher>;
    mod shared;
    pub use shared::{KeywordProcessor, Node};
}
