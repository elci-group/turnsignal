use crate::Signal;

pub fn render_advisory(signal: &Signal) -> String {
    format!(
        "→ Next: {}\nReason: {}\nConfidence: {:.2}",
        humanize(&signal.next.action),
        signal.next.reason,
        signal.confidence
    )
}

pub fn render_injection(signal: &Signal) -> String {
    format!(
        "[TURNSIGNAL]\nNext action: {}.\nReason: {}.\nConfidence: {:.2}.\n[/TURNSIGNAL]",
        humanize(&signal.next.action),
        trim_period(&signal.next.reason),
        signal.confidence
    )
}

pub fn render_readable(signal: &Signal) -> String {
    let route = signal
        .route
        .as_ref()
        .map(|route| format!("\nRoute: {} ({})", route.target, route.reason))
        .unwrap_or_default();
    format!(
        "TURN SIGNAL\n───────────\nState: {}\nOutcome: {}\nRecommended next action: {}\nActor: {}\nConfidence: {:.2}\nPriority: {}{}",
        signal.transition.to,
        signal.outcome,
        humanize(&signal.next.action),
        signal.next.actor,
        signal.confidence,
        signal.next.priority,
        route
    )
}

fn humanize(value: &str) -> String {
    value.replace('_', " ")
}

fn trim_period(value: &str) -> &str {
    value.strip_suffix('.').unwrap_or(value)
}
