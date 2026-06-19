use leptos::prelude::*;
use thaw::{Badge, BadgeAppearance, BadgeColor};

use crate::api_types::ServiceStatus;
use crate::i18n::{Locale, Msg, t};

#[component]
pub fn StatusBadge(status: Signal<ServiceStatus>, locale: Signal<Locale>) -> impl IntoView {
    view! {
        <Badge
            appearance=BadgeAppearance::Tint
            color=Signal::derive(move || {
                if status.get().active {
                    BadgeColor::Success
                } else {
                    BadgeColor::Danger
                }
            })
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
        </Badge>
    }
}
