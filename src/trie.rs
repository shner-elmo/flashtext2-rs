
pub mod case_sensitive {
    type HashMap<Node> = fxhash::FxHashMap<String, Node>;
    #[path = "/home/shneor/Desktop/projects/rust/flashtext2-rs/src/shared.rs"] pub mod shared;
    pub use shared::{Node, KeywordProcessor};
}

pub mod case_insensitive {
    type HashMap<Node> = case_insensitive_hashmap::CaseInsensitiveHashMap<Node>;
    #[path = "/home/shneor/Desktop/projects/rust/flashtext2-rs/src/shared.rs"] pub mod shared;
    pub use shared::{Node, KeywordProcessor};
}

