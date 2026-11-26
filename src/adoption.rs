//! Feature adoption curve: share of the eligible base that used a feature by each day since
//! launch, the day 50% was reached, and a weekly series for charts.
use crate::util::*;
use serde_json::{json, Value};

pub fn curve_op(i: &Value) -> R<Value> {
    let eligible = f(i, "eligible_accounts")?;
    if eligible <= 0.0 {
        return Err("eligible_accounts must be > 0".into());
    }
    let horizon = f_or(i, "horizon_days", 56.0) as i64;
    let mut out = vec![];
    for ft in arr(i, "features")? {
        let name = s(ft, "feature")?;
        let mut days: Vec<i64> = arr(ft, "first_use_days")?.iter().filter_map(Value::as_f64).map(|d| d as i64).filter(|d| *d >= 0).collect();
        if days.len() as f64 > eligible {
            return Err(format!("{name}: more adopters than eligible accounts"));
        }
        days.sort();
        let pct_at = |d: i64| days.iter().filter(|x| **x <= d).count() as f64 / eligible * 100.0;
        let half = days.get(((eligible / 2.0).ceil() as usize).saturating_sub(1)).copied().filter(|_| days.len() as f64 >= eligible / 2.0);
        let weekly: Vec<Value> = (0..=horizon / 7).map(|w| json!({"week": w, "adopted_pct": rd(pct_at(w * 7), 1)})).collect();
        out.push(json!({"feature": name, "adopted_pct": rd(pct_at(horizon), 1), "day_to_50pct": half, "median_first_use_day": days.get(days.len() / 2), "weekly": weekly}));
    }
    out.sort_by(|a, b| b["adopted_pct"].as_f64().unwrap().total_cmp(&a["adopted_pct"].as_f64().unwrap()));
    Ok(json!({"features": out}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn half_day() {
        let r = curve_op(&json!({"eligible_accounts":4,"horizon_days":14,"features":[{"feature":"x","first_use_days":[1,3,9]}]})).unwrap();
        assert_eq!(r["features"][0]["day_to_50pct"], 3);
        assert_eq!(r["features"][0]["adopted_pct"], 75.0);
        let r = curve_op(&json!({"eligible_accounts":10,"features":[{"feature":"x","first_use_days":[1]}]})).unwrap();
        assert!(r["features"][0]["day_to_50pct"].is_null());
    }
}
