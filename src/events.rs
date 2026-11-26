//! Segment-style `track` events to account features: event counts, distinct active users,
//! active days. Events with no account (`context.groupId`) are counted as orphans.
use crate::util::*;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub fn features_op(i: &Value) -> R<Value> {
    let mut acc: BTreeMap<String, (BTreeMap<String, u64>, BTreeSet<String>, BTreeSet<i64>)> = BTreeMap::new();
    let mut orphans = 0;
    let mut seen = BTreeSet::new();
    let mut dups = 0;
    for e in arr(i, "events")? {
        if s_or(e, "type", "track") != "track" {
            continue;
        }
        if let Some(id) = e.get("messageId").and_then(Value::as_str) {
            if !seen.insert(id.to_string()) {
                dups += 1;
                continue;
            }
        }
        let Some(g) = e.get("context").and_then(|c| c.get("groupId")).and_then(Value::as_str) else {
            orphans += 1;
            continue;
        };
        let x = acc.entry(g.to_string()).or_default();
        *x.0.entry(s(e, "event")?.to_string()).or_default() += 1;
        if let Some(u) = e.get("userId").and_then(Value::as_str) {
            x.1.insert(u.to_string());
        }
        x.2.insert(f(e, "day")? as i64);
    }
    let out: Map<String, Value> = acc
        .into_iter()
        .map(|(a, (ev, users, days))| (a, json!({"events": ev, "active_users": users.len(), "active_days": days.len()})))
        .collect();
    Ok(json!({"accounts": out, "orphan_events": orphans, "duplicate_events": dups}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn groups_and_dedupes() {
        let i = json!({"events":[{"messageId":"1","event":"x","userId":"u","day":1,"context":{"groupId":"A"}},
            {"messageId":"1","event":"x","userId":"u","day":1,"context":{"groupId":"A"}},
            {"event":"x","day":1},{"type":"identify","userId":"u"}]});
        let r = features_op(&i).unwrap();
        assert_eq!(r["accounts"]["A"]["events"]["x"], 1);
        assert_eq!((r["orphan_events"].as_u64(), r["duplicate_events"].as_u64()), (Some(1), Some(1)));
    }
}
