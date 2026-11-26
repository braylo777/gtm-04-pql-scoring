//! Stage 4 — PLG Telemetry & PQL Scoring.
//!
//! * `pql`      — Product Qualified Lead score = fit × usage, with capped, explainable
//!   feature points and a sales alert when an account crosses the threshold.
//! * `aha`      — find the "aha moment": the usage threshold that best separates retained
//!   from churned users, with a minimum group size.
//! * `adoption` — feature adoption curves from first-use days: cumulative %, day to 50%.
//! * `events`   — turn a Segment/RudderStack-style `track` stream into per-account features.

mod api;
pub mod adoption;
pub mod aha;
pub mod events;
pub mod pql;
pub mod util;

pub use api::dispatch;
use serde_json::Value;

pub const OPS: &[&str] = &["track_to_features", "pql_score", "aha_threshold", "adoption_curve"];

pub fn handle(op: &str, i: &Value) -> Result<Value, String> {
    match op {
        "track_to_features" => events::features_op(i),
        "pql_score" => pql::score_op(i),
        "aha_threshold" => aha::aha_op(i),
        "adoption_curve" => adoption::curve_op(i),
        _ => Err(format!("unknown op `{op}`; known: {}", OPS.join(", "))),
    }
}
