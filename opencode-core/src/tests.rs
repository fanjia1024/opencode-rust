#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::{PermissionAction, PermissionManager};
    use crate::session::{Message, MessageRole, Session};
    use crate::cache::{Cache, ConcurrentCache, ProviderCache};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_permission_manager() {
        let mut pm = PermissionManager::new();
        pm.add_rule("*.rs", PermissionAction::Allow).unwrap();
        pm.add_rule("*.ts", PermissionAction::Deny).unwrap();

        assert_eq!(pm.check("test.rs"), PermissionAction::Allow);
        assert_eq!(pm.check("test.ts"), PermissionAction::Deny);
        assert_eq!(pm.check("test.txt"), PermissionAction::Ask);
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            Uuid::new_v4().to_string(),
            "test-project".to_string(),
            "/tmp".to_string(),
        );
        assert!(!session.id.is_empty());
        assert_eq!(session.messages.len(), 0);
    }

    #[test]
    fn test_session_add_message() {
        let mut session = Session::new(
            Uuid::new_v4().to_string(),
            "test-project".to_string(),
            "/tmp".to_string(),
        );

        let message = Message {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: "Hello".to_string(),
            timestamp: Utc::now(),
        };

        session.add_message(message);
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn test_session_compaction() {
        let mut session = Session::new(
            Uuid::new_v4().to_string(),
            "test-project".to_string(),
            "/tmp".to_string(),
        );

        for i in 0..150 {
            let message = Message {
                id: Uuid::new_v4().to_string(),
                role: MessageRole::User,
                content: format!("Message {}", i),
                timestamp: Utc::now(),
            };
            session.add_message(message);
        }

        assert_eq!(session.messages.len(), 150);
        session.compact().unwrap();
        assert!(session.messages.len() <= 100);
    }

    #[tokio::test]
    async fn test_cache() {
        let cache: Cache<String, String> = Cache::new(10);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()).await, None);
    }

    #[test]
    fn test_concurrent_cache() {
        let cache: ConcurrentCache<String, String> = ConcurrentCache::new();
        
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_provider_cache_key() {
        use crate::agent::{ProviderRequest, Message, MessageRole};
        
        let request = ProviderRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "test".to_string(),
            }],
            model: Some("test".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(100),
        };
        
        let key1 = ProviderCache::cache_key_for_request(&request);
        let key2 = ProviderCache::cache_key_for_request(&request);
        assert_eq!(key1, key2);
    }
}
