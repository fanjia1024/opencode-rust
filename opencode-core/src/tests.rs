#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::{PermissionAction, PermissionManager};
    use crate::session::{Message, Role, Session};
    use crate::cache::{Cache, ConcurrentCache, ProviderCache};
    use chrono::Utc;

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
        let session = Session::new();
        assert_eq!(session.messages.len(), 0);
        assert!(session.is_empty());
    }

    #[test]
    fn test_session_add_message() {
        let mut session = Session::new();
        let message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            created_at: Utc::now(),
            meta: None,
        };
        session.push_message(message);
        assert_eq!(session.messages.len(), 1);
        assert!(!session.is_empty());
    }

    #[test]
    fn session_is_pure_data() {
        let s = Session::new();
        let json = serde_json::to_string(&s).unwrap();
        let _: Session = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn session_push_and_is_empty() {
        let mut session = Session::new();
        assert!(session.is_empty());
        session.push_message(Message {
            role: Role::User,
            content: "hi".to_string(),
            created_at: Utc::now(),
            meta: None,
        });
        assert!(!session.is_empty());
        assert_eq!(session.messages[0].content, "hi");
    }

    #[test]
    fn session_roundtrip() {
        let mut session = Session::new();
        session.push_message(Message {
            role: Role::User,
            content: "one".to_string(),
            created_at: Utc::now(),
            meta: None,
        });
        session.push_message(Message {
            role: Role::Assistant,
            content: "two".to_string(),
            created_at: Utc::now(),
            meta: None,
        });
        let json = serde_json::to_string(&session).unwrap();
        let loaded: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[0].content, "one");
        assert_eq!(loaded.messages[1].content, "two");
    }

    #[test]
    fn session_id_unique() {
        let s1 = Session::new();
        let s2 = Session::new();
        assert_ne!(s1.id, s2.id);
    }

    #[test]
    fn message_meta_optional() {
        let json = r#"{"role":"User","content":"x","created_at":"2024-01-01T00:00:00Z"}"#;
        let m: Message = serde_json::from_str(json).unwrap();
        assert!(m.meta.is_none());
        assert_eq!(m.content, "x");

        let with_meta = Message {
            role: Role::Tool,
            content: "y".to_string(),
            created_at: Utc::now(),
            meta: Some(crate::session::MessageMeta {
                tool_name: Some("read".to_string()),
                tool_call_id: Some("id1".to_string()),
            }),
        };
        let json2 = serde_json::to_string(&with_meta).unwrap();
        let m2: Message = serde_json::from_str(&json2).unwrap();
        assert!(m2.meta.is_some());
        assert_eq!(m2.meta.as_ref().unwrap().tool_name.as_deref(), Some("read"));
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
