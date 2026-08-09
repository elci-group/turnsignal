use crate::{NextAction, Observation, Route};

pub trait Router: Send + Sync {
    fn route(&self, observation: &Observation, next: &NextAction) -> Route;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultRouter;

impl Router for DefaultRouter {
    fn route(&self, observation: &Observation, next: &NextAction) -> Route {
        let text = format!("{} {}", observation.content, next.action).to_lowercase();
        let (target, reason) = if contains_any(&text, &["documentation", "docs", "readme"]) {
            ("goglz", "documentation work detected")
        } else if contains_any(&text, &["compliance", "policy", "audit"]) {
            ("isopod", "compliance work detected")
        } else if contains_any(&text, &["version", "release", "changelog"]) {
            ("kaptaind", "release work detected")
        } else if contains_any(&text, &["scalability", "capacity", "performance"]) {
            ("iris", "scalability work detected")
        } else if contains_any(
            &text,
            &["implement", "code", "test", "repository", "dependency"],
        ) {
            ("codex", "implementation work detected")
        } else {
            (next.actor.as_str(), "no specialist route matched")
        };

        Route {
            target: target.into(),
            reason: reason.into(),
        }
    }
}

fn contains_any(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| text.contains(term))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Priority;

    #[test]
    fn routes_release_work_to_kaptaind() {
        let observation = Observation::result("Prepare the version and release notes");
        let next = NextAction {
            action: "prepare_release".into(),
            actor: "main_agent".into(),
            priority: Priority::Normal,
            reason: "Implementation is verified".into(),
        };
        let route = DefaultRouter.route(&observation, &next);
        assert_eq!(route.target, "kaptaind");
    }
}
