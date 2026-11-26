//! Aha-moment search. For each candidate threshold k, split users into "did ≥ k" and "did < k"
//! within the first window, and compare retention. Pick the k with the largest lift where both
//! groups have at least `min_group` users. Correlation, not causation: say so in the output.
use crate::util::*;
use serde_json::{json, Value};

pub fn aha_op(i: &Value) -> R<Value> {
    let users = arr(i, "users")?;
    let min_g = f_or(i, "min_group", 5.0) as usize;
    let mut pts: Vec<(f64, bool)> = vec![];
    for u in users {
        let r = u.get("retained").and_then(Value::as_bool).ok_or("each user needs `retained` (bool)")?;
        pts.push((f(u, "actions_in_window")?, r));
    }
    let base = pts.iter().filter(|p| p.1).count() as f64 / pts.len().max(1) as f64;
    let mut ks: Vec<f64> = pts.iter().map(|p| p.0).collect();
    ks.sort_by(f64::total_cmp);
    ks.dedup();
    let mut table = vec![];
    let mut best: Option<(f64, f64, f64, f64, usize)> = None;
    for &k in &ks {
        let (hi, lo): (Vec<&(f64, bool)>, Vec<&(f64, bool)>) = pts.iter().partition(|p| p.0 >= k);
        if hi.len() < min_g || lo.len() < min_g {
            continue;
        }
        let rh = hi.iter().filter(|p| p.1).count() as f64 / hi.len() as f64;
        let rl = lo.iter().filter(|p| p.1).count() as f64 / lo.len() as f64;
        let lift = rh - rl;
        table.push(json!({"threshold": k, "users_at_or_above": hi.len(), "retention_at_or_above": rd(rh * 100.0, 1), "retention_below": rd(rl * 100.0, 1), "lift_pts": rd(lift * 100.0, 1)}));
        if best.map_or(true, |b| lift > b.1) {
            best = Some((k, lift, rh, rl, hi.len()));
        }
    }
    let b = best.ok_or("no threshold leaves both groups at or above min_group")?;
    Ok(json!({
        "aha_threshold": b.0,
        "statement": format!("Users who reach {} actions in the window retain at {:.1}% vs {:.1}% ({}/{} users reach it).", b.0, b.2 * 100.0, b.3 * 100.0, b.4, pts.len()),
        "baseline_retention": rd(base * 100.0, 1),
        "lift_pts": rd(b.1 * 100.0, 1),
        "candidates": table,
        "caution": "correlation only: confirm with an experiment that pushes users to the threshold",
    }))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn finds_separating_threshold() {
        let mut u = vec![];
        for k in 0..10 {
            u.push(json!({"actions_in_window": k, "retained": k >= 4}));
        }
        let r = aha_op(&json!({"users":u,"min_group":2})).unwrap();
        assert_eq!(r["aha_threshold"], 4.0);
        assert_eq!(r["lift_pts"], 100.0);
        assert!(aha_op(&json!({"users":[{"actions_in_window":1,"retained":true}],"min_group":5})).is_err());
    }
}
