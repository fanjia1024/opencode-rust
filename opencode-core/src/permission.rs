use crate::error::{Error, Result};
use globset::{Glob, GlobMatcher};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionAction {
    Allow,
    Deny,
    Ask,
}

pub struct PermissionManager {
    rules: HashMap<String, PermissionAction>,
    matchers: Vec<(GlobMatcher, PermissionAction)>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            matchers: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, pattern: &str, action: PermissionAction) -> Result<()> {
        let glob = Glob::new(pattern)
            .map_err(|e| Error::Validation(format!("Invalid glob pattern {}: {}", pattern, e)))?;
        let matcher = glob.compile_matcher();
        self.matchers.push((matcher, action.clone()));
        self.rules.insert(pattern.to_string(), action);
        Ok(())
    }

    pub fn check(&self, resource: &str) -> PermissionAction {
        for (matcher, action) in &self.matchers {
            if matcher.is_match(resource) {
                return action.clone();
            }
        }
        PermissionAction::Ask
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
