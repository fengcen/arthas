
use std::collections::HashMap;
use std::iter::IntoIterator;
use std::cmp::Eq;
use std::hash::Hash;


pub fn revert<K, V>(hash_map: HashMap<K, V>) -> HashMap<V, K>
    where HashMap<K, V>: IntoIterator<Item = (K, V)>,
          V: Eq + Hash
{
    let mut reverted_map = HashMap::new();

    for (k, v) in hash_map {
        reverted_map.insert(v, k);
    }

    reverted_map
}
