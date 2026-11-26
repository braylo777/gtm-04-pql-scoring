//! PQL scoring. Usage points: each feature gives `points_per_unit` per unit up to `max_points`.
//! Fit points: firmographic rules. Score = usage (0–100) × fit multiplier. Every point has a reason.
use crate::util::*;
use serde_json::{json, Value};

pub fn score_op(i: &Value) -> R<Value> {
    let feats = arr(i, "usage_model")?;
    let fit = arr(i, "fit_model")?;
    let thr = f(i, "pql_threshold")?;
    let total_max: f64 = feats.iter().map(|x| f_or(x, "max_points", 0.0)).sum();
    if total_max <= 0.0 {
        return Err("usage_model max_points must add up to > 0".into());
    }
    let mut out = vec![];
    for a in arr(i, "accounts")? {
        let id = s(a, "account")?;
        let mut usage = 0.0;
        let mut reasons = vec![];
        for ft in feats {
            let name = s(ft, "feature")?;
            let v = a.get("usage").and_then(|u| u.get(name)).and_then(Value::as_f64).unwrap_or(0.0);
            let p = (v * f(ft, "points_per_unit")?).min(f(ft, "max_points")?).max(0.0);
            if p > 0.0 {
                reasons.push(format!("{name}: {v} → +{}", rd(p, 1)));
            }
            usage += p;
        }
        let usage_pct = usage / total_max * 100.0;
        let mut mult = 1.0;
        for r in fit {
            let field = s(r, "field")?;
            let v = a.get(field);
            let ok = match (r.get("in"), r.get("min"), r.get("max")) {
                (Some(set), _, _) => set.as_array().map_or(false, |s| v.map_or(false, |v| s.contains(v))),
                (_, mn, mx) => v.and_then(Value::as_f64).map_or(false, |x| mn.and_then(Value::as_f64).map_or(true, |m| x >= m) && mx.and_then(Value::as_f64).map_or(true, |m| x <= m)),
            };
            let m = if ok { f_or(r, "multiplier_if_true", 1.0) } else { f_or(r, "multiplier_if_false", 1.0) };
            if m != 1.0 {
                reasons.push(format!("fit {field} {} → ×{m}", if ok { "match" } else { "miss" }));
            }
            mult *= m;
        }
        let score = (usage_pct * mult).min(100.0);
        let pql = score >= thr;
        out.push(json!({"account": id, "score": rd(score, 1), "usage_pct": rd(usage_pct, 1), "fit_multiplier": rd(mult, 3),
            "is_pql": pql, "action": if pql {"alert_ae"} else if score >= thr * 0.7 {"nurture_in_app"} else {"self_serve"}, "reasons": reasons}));
    }
    out.sort_by(|a, b| b["score"].as_f64().unwrap().total_cmp(&a["score"].as_f64().unwrap()));
    let n = out.iter().filter(|x| x["is_pql"] == true).count();
    Ok(json!({"pql_count": n, "accounts": out}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn caps_and_fit() {
        let i = json!({"pql_threshold":50,"usage_model":[{"feature":"seats","points_per_unit":10,"max_points":50},{"feature":"api","points_per_unit":1,"max_points":50}],
            "fit_model":[{"field":"employees","min":50,"multiplier_if_false":0.5}],
            "accounts":[{"account":"A","employees":200,"usage":{"seats":99,"api":10}},{"account":"B","employees":5,"usage":{"seats":99,"api":50}}]});
        let r = score_op(&i).unwrap();
        assert_eq!(r["accounts"][0]["account"], "A");
        assert_eq!(r["accounts"][0]["score"], 60.0);
        assert_eq!(r["accounts"][1]["score"], 50.0);
        assert_eq!(r["pql_count"], 2);
    }
}
