use opentelemetry::{KeyValue, metrics::UpDownCounter};
use std::sync::LazyLock;

static STATDS: LazyLock<UpDownCounter<i64>> = LazyLock::new(|| {
    logfire::i64_up_down_counter("pet_info_statds")
        .with_description("Pet info app statistics")
        .with_unit("attempt")
        .build()
});

fn incr_statds(metric: String, value: String) {
    STATDS.add(1, &[KeyValue::new(metric, value)]);
}

pub fn incr_user_action_statds(action: &str) {
    incr_statds("user_action".to_string(), action.into())
}

pub fn incr_payment_status_statds(status: &str) {
    incr_statds("payment_status".to_string(), status.into())
}

pub fn incr_reminder_action_statds(action: &str) {
    incr_statds("reminder".to_string(), action.into())
}
