use leptos::prelude::*;

use crate::api_types::ServiceStatus;
use crate::i18n::{Locale, Msg, t};

#[component]
pub fn status_badge(status: Signal<ServiceStatus>, locale: Signal<Locale>) -> impl IntoView {
    view! {
        <span
            class=move || if status.with(|status| status.active) {
                "status-badge status-badge--active"
            } else {
                "status-badge status-badge--inactive"
            }
            role="status"
        >
            {move || {
                let status = status.get();
                if status.active {
                    t(locale.get(), Msg::Active).into()
                } else if status.description.is_empty() {
                    t(locale.get(), Msg::Inactive).into()
                } else {
                    status.description
                }
            }}
        </span>
    }
}
