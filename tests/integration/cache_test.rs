#[cfg(test)]
mod tests {
    use opencode_core::cache::{Cache, ConcurrentCache, ProviderCache};

    #[tokio::test]
    async fn test_lru_cache() {
        let cache: Cache<String, String> = Cache::new(3);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.put("key2".to_string(), "value2".to_string()).await;
        cache.put("key3".to_string(), "value3".to_string()).await;
        
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        assert_eq!(cache.len().await, 3);
        
        cache.put("key4".to_string(), "value4".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, None);
        assert_eq!(cache.get(&"key4".to_string()).await, Some("value4".to_string()));
    }

    #[test]
    fn test_concurrent_cache() {
        use std::sync::Arc;
        use std::thread;
        
        let cache: Arc<ConcurrentCache<String, usize>> = Arc::new(ConcurrentCache::new());
        
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cache = cache.clone();
                thread::spawn(move || {
                    for j in 0..100 {
                        let key = format!("key_{}", i);
                        cache.insert(key.clone(), j);
                        assert_eq!(cache.get(&key), Some(j));
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert!(cache.len() > 0);
    }

    #[test]
    fn test_provider_cache() {
        let cache = ProviderCache::new();
        
        cache.cache_response("key1".to_string(), "response1".to_string());
        assert_eq!(cache.get_response("key1"), Some("response1".to_string()));
        assert_eq!(cache.get_response("key2"), None);
    }
}
