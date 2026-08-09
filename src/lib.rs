//! Turnsignal converts verbose agent activity into compact state transitions.

mod engine;
mod model;
mod render;
mod router;

pub use engine::{Inferencer, RuleBasedInferencer, Turnsignal};
pub use model::{
    Delivery, Evidence, Injection, InjectionMode, NextAction, Observation, ObservationKind,
    Priority, Route, Signal, SignalMode, Transition,
};
pub use render::{render_advisory, render_injection, render_readable};
pub use router::{DefaultRouter, Router};
