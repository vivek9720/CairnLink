use crate::catalog;
use crate::session::SessionState;

#[derive(Clone, Debug, Default)]
pub struct RiskReport {
    pub score: i64,
    pub messages: Vec<String>,
}

pub fn analyze_session(state: &mut SessionState) -> RiskReport {
    let mut score = 0i64;
    let mut messages = Vec::new();

    if let Some(contact) = state.manifest.primary_contact() {
        score += contact.len() as i64;
        if contact.contains("ops") {
            messages.push(format!("field contact {}", contact));
        }
    }

    score ^= state.manifest.anchor_score() as i64;
    score = score.saturating_add(state.dictionary.audit_aliases() as i64);
    score = score.saturating_add(state.routes.score_cached_lane() as i64);
    score ^= state.routes.audit_labels() as i64;
    score = score.saturating_add(state.inventory.reconcile() as i64);
    state.fragments.materialize_pending();
    state.reports.render();

    for point in &state.telemetry {
        let profile = catalog::shelter_profile(point.shelter_id as usize);
        if point.occupancy > profile.capacity {
            score += (point.occupancy - profile.capacity) as i64 * 3;
        }
    }

    for summary in &state.journal_summaries {
        score = score.saturating_add(summary.score);
        score = score.saturating_add(summary.events as i64);
    }

    for script in &state.script_results {
        score = score.saturating_add(script.accumulator);
        score = score.saturating_add(script.alerts as i64 * 11);
    }

    if state.reports.rendered_count() > 0 {
        messages.push(format!(
            "rendered {} reports",
            state.reports.rendered_count()
        ));
    }

    RiskReport { score, messages }
}
