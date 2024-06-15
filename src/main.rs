// use std::collections::hash_map::Entry;
// use unicase::UniCase;
//
// #[derive(Debug, Default)]
// struct HashMap<K, V> {
//     inner: std::collections::HashMap<UniCase<K>, V, fxhash::FxBuildHasher>,
// }
//
// impl<K, V> HashMap<K, V>
// where
//     K: AsRef<str>,
// {
//     pub fn entry<Q: Into<UniCase<K>>>(&mut self, k: Q) -> Entry<UniCase<K>, V> {
//         self.inner.entry(k.into())
//     }
//
//     pub fn get<Q: Into<UniCase<K>>>(&self, k: Q) -> Option<&V> {
//         self.inner.get(&k.into())
//     }
// }
//
// // pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
// // where
// //     K: Borrow<Q>,
// //     Q: Hash + Eq + AsRef<str>,
// // {
// //     self.inner.get(k)
// // }
//
// fn main() {
//     let mut a: HashMap<&str, u32> = HashMap::default();
//     a.entry("fes").or_insert(323);
//     assert_eq!(a.get("FeS"), Some(&323));
//     assert_eq!(a.get("FeS"), Some(&323));
//
//     let mut a: HashMap<String, u32> = HashMap::default();
//     a.entry("fes".to_string()).or_insert(323);
//     assert_eq!(a.get("FeS".to_string()), Some(&323));
//     assert_eq!(a.get("FeS"), Some(&323));
//
//     let mut a: HashMap<String, u32> = HashMap::default();
//     a.entry("fes".to_string()).or_insert(323);
//     assert_eq!(a.get("FeS"), Some(&323));
//     assert_eq!(a.get("FES"), Some(&323));
//     assert_eq!(a.get("fes"), Some(&323));
//     // a.inner.insert("fes", 3);
//     // a.inner.insert(UniCase::new(""), 3);
//     // a.inner.insert(UniCase::unicode("Maße"), 3);
//     // a.inner.insert("Maße".into(), 3);
//     // assert!(a.inner.contains_key(&UniCase::new("MASSE")));
//     // assert!(a.inner.contains_key(&UniCase::unicode("MASSE")));
//     // assert!(a.inner.contains_key(&"MASSE".into()));
// }
fn main() {}
