use crate::model::{
    Delivery, Evidence, Injection, InjectionMode, NextAction, Observation, ObservationKind,
    Priority, Signal, SignalMode, Transition,
};
use crate::render::{render_advisory, render_injection, render_readable};
use crate::router::{DefaultRouter, Router};

#[derive(Debug, Clone, PartialEq)]
pub struct Inference {
    pub state: String,
    pub outcome: String,
    pub confidence: f32,
    pub action: String,
    pub actor: String,
    pub priority: Priority,
    pub reason: String,
    pub evidence: Vec<String>,
}

pub trait Inferencer: Send + Sync {
    fn infer(&self, observation: &Observation) -> Inference;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuleBasedInferencer;

impl Inferencer for RuleBasedInferencer {
    fn infer(&self, observation: &Observation) -> Inference {
        let text = observation.content.to_lowercase();

        if observation.kind == ObservationKind::Projected {
            return infer_projection(&text);
        }

        if contains_any(
            &text,
            &["failed", "failure", "error", "blocked", "cannot", "can't"],
        ) {
            inference(
                "operation_failed",
                "blocked",
                0.92,
                "diagnose_failure",
                Priority::Critical,
                "The previous operation did not complete successfully",
                evidence(
                    &observation.content,
                    &["fail", "error", "block", "cannot", "can't"],
                ),
            )
        } else if tests_missing(&text) {
            inference(
                "repository_analysis_complete",
                "test_gap_identified",
                0.91,
                "inspect_test_infrastructure",
                Priority::High,
                "Tests or compliance checks appear to be absent",
                evidence(
                    &observation.content,
                    &["test", "compliance", "absent", "missing"],
                ),
            )
        } else if tests_passed(&text) {
            inference(
                "verification_complete",
                "checks_passed",
                0.96,
                "summarize_and_handoff",
                Priority::Normal,
                "The implementation has passed its verification gate",
                evidence(
                    &observation.content,
                    &["pass", "green", "success", "verified"],
                ),
            )
        } else if contains_any(
            &text,
            &[
                "implemented",
                "implementation complete",
                "code complete",
                "fixed",
            ],
        ) {
            inference(
                "implementation_complete",
                "change_ready_for_verification",
                0.88,
                "run_relevant_tests",
                Priority::High,
                "A completed change should be verified before handoff",
                evidence(&observation.content, &["implement", "complete", "fixed"]),
            )
        } else if contains_any(
            &text,
            &[
                "inspected the repository",
                "repository inspected",
                "architecture understood",
            ],
        ) {
            inference(
                "repository_inspected",
                "architecture_understood",
                0.84,
                "identify_highest_risk_gap",
                Priority::Normal,
                "Repository discovery is complete; the next constraint should be selected",
                evidence(
                    &observation.content,
                    &["repository", "architecture", "module"],
                ),
            )
        } else if contains_any(
            &text,
            &["needs approval", "awaiting approval", "permission required"],
        ) {
            inference(
                "approval_required",
                "external_authority_needed",
                0.95,
                "request_authorization",
                Priority::High,
                "Progress requires authority not currently available",
                evidence(
                    &observation.content,
                    &["approval", "permission", "authority"],
                ),
            )
        } else {
            inference(
                "outcome_observed",
                "next_step_undetermined",
                0.52,
                "review_and_choose_next_action",
                Priority::Normal,
                "No high-confidence transition rule matched the observation",
                evidence(&observation.content, &[]),
            )
        }
    }
}

pub struct Turnsignal<I = RuleBasedInferencer, R = DefaultRouter> {
    inferencer: I,
    router: R,
}

impl Default for Turnsignal {
    fn default() -> Self {
        Self::new(RuleBasedInferencer, DefaultRouter)
    }
}

impl<I, R> Turnsignal<I, R>
where
    I: Inferencer,
    R: Router,
{
    pub fn new(inferencer: I, router: R) -> Self {
        Self { inferencer, router }
    }

    pub fn observe(&self, observation: &Observation, previous_state: Option<&str>) -> Signal {
        let inferred = self.inferencer.infer(observation);
        let next = NextAction {
            action: inferred.action,
            actor: inferred.actor,
            priority: inferred.priority,
            reason: inferred.reason,
        };

        Signal {
            source: observation.source.clone(),
            transition: Transition {
                from: previous_state.map(str::to_owned),
                to: inferred.state,
            },
            outcome: inferred.outcome,
            confidence: inferred.confidence.clamp(0.0, 1.0),
            evidence: Evidence {
                items: inferred.evidence,
            },
            injection: Injection {
                mode: InjectionMode::Context,
                ttl: 1,
            },
            next,
            route: None,
        }
    }

    pub fn dispatch(
        &self,
        observation: &Observation,
        previous_state: Option<&str>,
        mode: SignalMode,
    ) -> Delivery {
        let mut signal = self.observe(observation, previous_state);
        if mode == SignalMode::Routing {
            signal.route = Some(self.router.route(observation, &signal.next));
        }
        let message = match mode {
            SignalMode::Advisory => render_advisory(&signal),
            SignalMode::Injective => render_injection(&signal),
            SignalMode::Routing => render_readable(&signal),
        };
        Delivery {
            mode,
            signal,
            message,
        }
    }
}

fn infer_projection(text: &str) -> Inference {
    if contains_any(text, &["redis", "cache", "caching"])
        || (contains_any(text, &["add", "introduce"])
            && contains_any(text, &["dependency", "service", "database"]))
    {
        inference(
            "cross_cutting_change_projected",
            "operational_consequences_anticipated",
            0.90,
            "inspect_existing_abstractions_and_deployment",
            Priority::High,
            "The intended change crosses dependency, configuration, and operations boundaries",
            vec![
                "dependency addition likely".into(),
                "configuration and deployment changes likely".into(),
                "new failure and invalidation modes likely".into(),
            ],
        )
    } else {
        inference(
            "change_projected",
            "impact_analysis_required",
            0.68,
            "inspect_affected_boundaries",
            Priority::Normal,
            "The intended action should be checked for downstream effects",
            evidence(text, &[]),
        )
    }
}

fn inference(
    state: &str,
    outcome: &str,
    confidence: f32,
    action: &str,
    priority: Priority,
    reason: &str,
    evidence: Vec<String>,
) -> Inference {
    Inference {
        state: state.into(),
        outcome: outcome.into(),
        confidence,
        action: action.into(),
        actor: "main_agent".into(),
        priority,
        reason: reason.into(),
        evidence,
    }
}

fn tests_missing(text: &str) -> bool {
    contains_any(
        text,
        &["no tests", "tests absent", "tests missing", "without tests"],
    ) || (text.contains("compliance") && contains_any(text, &["absent", "missing", "none"]))
}

fn tests_passed(text: &str) -> bool {
    (contains_any(text, &["test", "check", "verification"])
        && contains_any(text, &["passed", "pass", "green", "successful"]))
        || text.contains("all tests pass")
}

fn contains_any(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| text.contains(term))
}

fn evidence(content: &str, terms: &[&str]) -> Vec<String> {
    let sentences: Vec<String> = content
        .split(['\n', '.', ';'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            terms.is_empty() || terms.iter().any(|term| line.to_lowercase().contains(term))
        })
        .take(3)
        .map(|line| line.chars().take(180).collect())
        .collect();

    if sentences.is_empty() {
        vec![content.chars().take(180).collect()]
    } else {
        sentences
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_missing_test_infrastructure() {
        let engine = Turnsignal::default();
        let observation = Observation::result(
            "Inspected the repository. Rust daemon with 17 modules. Compliance tests absent.",
        );
        let signal = engine.observe(&observation, Some("discovery_started"));
        assert_eq!(signal.transition.from.as_deref(), Some("discovery_started"));
        assert_eq!(signal.transition.to, "repository_analysis_complete");
        assert_eq!(signal.next.action, "inspect_test_infrastructure");
        assert!(signal.confidence >= 0.9);
    }

    #[test]
    fn anticipates_cross_cutting_cache_work() {
        let engine = Turnsignal::default();
        let observation = Observation::projected("Add Redis caching to the API");
        let signal = engine.observe(&observation, None);
        assert_eq!(signal.transition.to, "cross_cutting_change_projected");
        assert_eq!(signal.evidence.items.len(), 3);
    }

    #[test]
    fn routes_compliance_to_specialist() {
        let engine = Turnsignal::default();
        let observation = Observation::result("The compliance audit is ready for review");
        let delivery = engine.dispatch(&observation, None, SignalMode::Routing);
        assert_eq!(delivery.signal.route.unwrap().target, "isopod");
    }

    #[test]
    fn injection_is_short_lived_and_compact() {
        let engine = Turnsignal::default();
        let observation = Observation::result("Implementation complete");
        let delivery = engine.dispatch(&observation, None, SignalMode::Injective);
        assert_eq!(delivery.signal.injection.ttl, 1);
        assert!(delivery.message.starts_with("[TURNSIGNAL]"));
        assert!(delivery.message.len() < 300);
    }

    #[test]
    fn failures_take_precedence_over_success_terms() {
        let engine = Turnsignal::default();
        let observation = Observation::result("Tests passed, but deployment failed");
        let signal = engine.observe(&observation, None);
        assert_eq!(signal.transition.to, "operation_failed");
        assert_eq!(signal.next.priority, Priority::Critical);
    }
}
