use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    #[default]
    Result,
    Projected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub kind: ObservationKind,
    pub content: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

fn default_source() -> String {
    "agent".into()
}

impl Observation {
    pub fn result(content: impl Into<String>) -> Self {
        Self {
            source: default_source(),
            kind: ObservationKind::Result,
            content: content.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn projected(content: impl Into<String>) -> Self {
        Self {
            kind: ObservationKind::Projected,
            ..Self::result(content)
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SignalMode {
    #[default]
    Advisory,
    Injective,
    Routing,
}

impl FromStr for SignalMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "advisory" => Ok(Self::Advisory),
            "injective" => Ok(Self::Injective),
            "routing" => Ok(Self::Routing),
            _ => Err(format!("unknown mode `{value}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextAction {
    pub action: String,
    pub actor: String,
    pub priority: Priority,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionMode {
    Context,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Injection {
    pub mode: InjectionMode,
    pub ttl: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub target: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    pub source: String,
    pub transition: Transition,
    pub outcome: String,
    pub confidence: f32,
    pub next: NextAction,
    pub evidence: Evidence,
    pub injection: Injection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Route>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delivery {
    pub mode: SignalMode,
    pub signal: Signal,
    pub message: String,
}
