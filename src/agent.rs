use rand::seq::SliceRandom;
use std::sync::LazyLock;

#[derive(Debug, Clone, serde::Deserialize)]
struct AgentData {
    pub patch: String,
    pub agents: Vec<Agent>,
}

static AGENT_DATA: LazyLock<AgentData> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/agents.json"))
        .expect("failed to parse agents.json")
});

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Agent {
    pub name: String,
    pub id: String,
    pub uuid: String,
    pub role: Role,
}

impl Agent {
    pub fn tracker_url(&self) -> String {
        format!("https://tracker.gg/valorant/db/agents/{}", self.id)
    }

    pub fn icon_url(&self) -> String {
        format!(
            "https://media.valorant-api.com/agents/{}/displayicon.png",
            self.uuid
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, poise::ChoiceParameter)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Controller,
    Duelist,
    Initiator,
    Sentinel,
}

pub fn patch() -> &'static str {
    &AGENT_DATA.patch
}

pub fn all_agents() -> &'static [Agent] {
    &AGENT_DATA.agents
}

pub fn agent_names() -> Vec<String> {
    AGENT_DATA.agents.iter().map(|a| a.name.clone()).collect()
}

pub fn filter_agents(role: Option<Role>, ignore: &[String]) -> Vec<&'static Agent> {
    AGENT_DATA
        .agents
        .iter()
        .filter(|a| role.is_none_or(|r| a.role == r))
        .filter(|a| !ignore.iter().any(|name| a.name.eq_ignore_ascii_case(name)))
        .collect()
}

pub fn pick_random<'a>(agents: &[&'a Agent], count: usize) -> Vec<&'a Agent> {
    let mut rng = rand::thread_rng();
    agents.choose_multiple(&mut rng, count).copied().collect()
}
