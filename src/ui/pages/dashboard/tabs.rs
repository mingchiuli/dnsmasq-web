use leptos::prelude::*;

use crate::i18n::{Locale, Msg, t};

pub(super) const TAB_ADDRESS: &str = "address";
pub(super) const TAB_HOST_RECORD: &str = "host-record";
pub(super) const TAB_CNAME: &str = "cname";
pub(super) const TAB_SERVER: &str = "server";
pub(super) const TAB_RAW: &str = "raw";
pub(super) const TAB_BACKUPS: &str = "backups";

const TAB_ITEMS: [(&str, Msg); 6] = [
    (TAB_ADDRESS, Msg::Address),
    (TAB_HOST_RECORD, Msg::HostRecord),
    (TAB_CNAME, Msg::Cname),
    (TAB_SERVER, Msg::Server),
    (TAB_RAW, Msg::RawConfig),
    (TAB_BACKUPS, Msg::Backups),
];

#[component]
pub(super) fn dashboard_tabs(
    active_tab: RwSignal<String>,
    locale: Signal<Locale>,
) -> impl IntoView {
    view! {
        <div class="tabs dashboard-tabs" role="tablist">
            <For
                each=move || TAB_ITEMS
                key=|item| item.0
                children=move |(value, message)| {
                    view! {
                        <button
                            id=format!("tab-{value}")
                            class=move || if active_tab.with(|active| active == value) {
                                "dashboard-tab dashboard-tab--active"
                            } else {
                                "dashboard-tab"
                            }
                            type="button"
                            role="tab"
                            aria-selected=move || if active_tab.with(|active| active == value) {
                                "true"
                            } else {
                                "false"
                            }
                            aria-controls=format!("panel-{value}")
                            tabindex=move || if active_tab.with(|active| active == value) { 0 } else { -1 }
                            on:click=move |_| active_tab.set(value.into())
                            on:keydown=move |event| {
                                if let Some(next) = tab_for_key(value, &event.key()) {
                                    event.prevent_default();
                                    active_tab.set(next.into());
                                    focus_tab(next);
                                }
                            }
                        >
                            {move || t(locale.get(), message)}
                        </button>
                    }
                }
            />
        </div>
    }
}

#[component]
pub(super) fn dashboard_tab_panel(
    value: &'static str,
    active_tab: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            id=format!("panel-{value}")
            role="tabpanel"
            aria-labelledby=format!("tab-{value}")
            hidden=move || active_tab.with(|active| active != value)
        >
            {children()}
        </div>
    }
}

fn tab_for_key(current: &str, key: &str) -> Option<&'static str> {
    let index = TAB_ITEMS.iter().position(|(value, _)| *value == current)?;
    let next = match key {
        "ArrowRight" => (index + 1) % TAB_ITEMS.len(),
        "ArrowLeft" => (index + TAB_ITEMS.len() - 1) % TAB_ITEMS.len(),
        "Home" => 0,
        "End" => TAB_ITEMS.len() - 1,
        _ => return None,
    };
    Some(TAB_ITEMS[next].0)
}

#[cfg(feature = "hydrate")]
fn focus_tab(value: &str) {
    use leptos::wasm_bindgen::JsCast;

    let Some(element) = document().get_element_by_id(&format!("tab-{value}")) else {
        return;
    };
    let Some(element) = element.dyn_ref::<leptos::web_sys::HtmlElement>() else {
        return;
    };
    let _ = element.focus();
}

#[cfg(not(feature = "hydrate"))]
fn focus_tab(_value: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_navigation_moves_and_wraps() {
        assert_eq!(
            tab_for_key(TAB_ADDRESS, "ArrowRight"),
            Some(TAB_HOST_RECORD)
        );
        assert_eq!(tab_for_key(TAB_ADDRESS, "ArrowLeft"), Some(TAB_BACKUPS));
        assert_eq!(tab_for_key(TAB_BACKUPS, "ArrowRight"), Some(TAB_ADDRESS));
        assert_eq!(tab_for_key(TAB_SERVER, "Home"), Some(TAB_ADDRESS));
        assert_eq!(tab_for_key(TAB_SERVER, "End"), Some(TAB_BACKUPS));
        assert_eq!(tab_for_key(TAB_SERVER, "Enter"), None);
    }
}
