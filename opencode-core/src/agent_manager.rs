use crate::agent::{Agent, AgentMode, BuildAgent, Context, GeneralAgent, PlanAgent, Provider};
use crate::error::Result;
use crate::session::Session;
use crate::tool::Tool;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AgentManager {
    agents: HashMap<String, Arc<dyn Agent>>,
    current_agent: String,
}

impl AgentManager {
    pub fn new() -> Self {
        let mut agents = HashMap::new();
        agents.insert("build".to_string(), Arc::new(BuildAgent::new()) as Arc<dyn Agent>);
        agents.insert("plan".to_string(), Arc::new(PlanAgent::new()) as Arc<dyn Agent>);
        agents.insert("general".to_string(), Arc::new(GeneralAgent::new()) as Arc<dyn Agent>);

        Self {
            agents,
            current_agent: "build".to_string(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Agent>> {
        self.agents.get(name)
    }

    pub fn current(&self) -> Option<&Arc<dyn Agent>> {
        self.agents.get(&self.current_agent)
    }

    pub fn switch(&mut self, name: &str) -> Result<()> {
        if self.agents.contains_key(name) {
            self.current_agent = name.to_string();
            Ok(())
        } else {
            Err(crate::error::Error::Agent(format!("Agent not found: {}", name)))
        }
    }

    pub async fn process(
        &self,
        ctx: &Context,
        input: &str,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &[Arc<dyn Tool>],
    ) -> Result<()> {
        let agent = self.current()
            .ok_or_else(|| crate::error::Error::Agent("No current agent".to_string()))?;
        
        agent.process(ctx, input, session, provider, tools).await
    }

    pub fn list(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    pub fn current_name(&self) -> &str {
        &self.current_agent
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}
