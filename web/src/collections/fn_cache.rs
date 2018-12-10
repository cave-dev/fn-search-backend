use fn_search_backend_db::models::Function;
use radix_trie::{Trie, TrieCommon};
use std::iter::FromIterator;

pub struct FnCache {
    trie: Trie<String, Vec<i64>>,
}

impl FnCache {
    fn new() -> Self {
        FnCache { trie: Trie::new() }
    }

    /// returns at most num function ids with signature sig, starting at index starting_index
    pub fn search(&self, sig: &str, num: usize, starting_index: Option<usize>) -> Option<&[i64]> {
        if let Some(cache) = self.trie.get(sig) {
            let start = if let Some(s) = starting_index { s } else { 0 };
            let len = cache.len();
            if start >= len {
                return None;
            }
            let end = if len < start + num { len } else { start + num };
            Some(&cache[start..end])
        } else {
            None
        }
    }

    /// returns at most num suggested type signature ids for completing sig
    pub fn suggest(&self, sig: &str, num: usize) -> Option<Vec<&str>> {
        if let Some(t) = self.trie.get_raw_descendant(sig) {
            let mut res = Vec::new();
            for k in (&t).keys().take(num) {
                res.push(k.as_str());
            }
            if res.is_empty() {
                None
            } else {
                Some(res)
            }
        } else {
            None
        }
    }

    // ASSUME EACH FUNCTION IS ONLY INSERTED ONCE!!!
    fn insert(&mut self, type_signature: &str, func_id: i64) {
        self.trie.map_with_default(
            type_signature.to_string(),
            |cache| {
                cache.push(func_id);
            },
            [func_id].to_vec(),
        );
    }
}

impl FromIterator<(String, i64)> for FnCache {
    fn from_iter<T: IntoIterator<Item = (String, i64)>>(fns: T) -> Self {
        let mut c = FnCache::new();
        for f in fns {
            c.insert(f.0.as_str(), f.1);
        }
        c
    }
}

impl<'a> FromIterator<(&'a str, i64)> for FnCache {
    fn from_iter<T: IntoIterator<Item = (&'a str, i64)>>(fns: T) -> Self {
        let mut c = FnCache::new();
        for f in fns {
            c.insert(f.0, f.1);
        }
        c
    }
}

impl FromIterator<Function> for FnCache {
    fn from_iter<T: IntoIterator<Item = Function>>(fns: T) -> Self {
        let mut c = FnCache::new();
        for f in fns {
            c.insert(f.type_signature.as_str(), f.id);
        }
        c
    }
}

impl<'a> FromIterator<&'a Function> for FnCache {
    fn from_iter<T: IntoIterator<Item = &'a Function>>(fns: T) -> Self {
        let mut c = FnCache::new();
        for f in fns {
            c.insert(f.type_signature.as_str(), f.id);
        }
        c
    }
}
