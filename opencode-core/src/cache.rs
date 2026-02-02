use crate::error::Result;
use dashmap::DashMap;
use lru::LruCache;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Cache<K, V> {
    cache: Arc<RwLock<LruCache<K, V>>>,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap();
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        cache.get(key).cloned()
    }

    pub async fn put(&self, key: K, value: V) {
        let mut cache = self.cache.write().await;
        cache.put(key, value);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

pub struct ConcurrentCache<K, V> {
    cache: DashMap<K, V>,
}

impl<K, V> ConcurrentCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).map(|v| v.clone())
    }

    pub fn insert(&self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.cache.remove(key).map(|(_, v)| v)
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

impl<K, V> Default for ConcurrentCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProviderCache {
    cache: ConcurrentCache<String, String>,
}

impl ProviderCache {
    pub fn new() -> Self {
        Self {
            cache: ConcurrentCache::new(),
        }
    }

    pub fn get_response(&self, key: &str) -> Option<String> {
        self.cache.get(&key.to_string())
    }

    pub fn cache_response(&self, key: String, value: String) {
        self.cache.insert(key, value);
    }

    pub fn cache_key_for_request(request: &crate::agent::ProviderRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        let key_str = format!("{:?}", request);
        key_str.hash(&mut hasher);
        format!("provider:{}", hasher.finish())
    }
}

impl Default for ProviderCache {
    fn default() -> Self {
        Self::new()
    }
}
