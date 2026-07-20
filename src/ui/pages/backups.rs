use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue;

use crate::api_types::BackupInfo;
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::button::{Button, ButtonSize, ButtonVariant};

#[component]
pub fn backups_panel(
    backups: Signal<Vec<BackupInfo>>,
    #[prop(into)] on_refresh: Callback<()>,
    #[prop(into)] on_restore: Callback<String>,
    #[prop(into)] on_delete: Callback<String>,
    locale: Signal<Locale>,
) -> impl IntoView {
    view! {
        <section class="backups">
            <div class="section-head">
                <h2>{move || t(locale.get(), Msg::Backups)}</h2>
                <Button on_click=move |_| on_refresh.run(())>
                    {move || t(locale.get(), Msg::Refresh)}
                </Button>
            </div>
            <div class="record-table">
                <table class="ui-table">
                    <thead>
                        <tr>
                            <th scope="col">{move || t(locale.get(), Msg::BackupId)}</th>
                            <th scope="col">{move || t(locale.get(), Msg::BackupSize)}</th>
                            <th scope="col">"Path"</th>
                            <th scope="col" class="actions-col">{move || t(locale.get(), Msg::Actions)}</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || backups.get()
                            key=|backup| backup.id.clone()
                            children=move |backup| {
                                let backup_id = backup.id;
                                let path = backup.path;
                                let size = backup.size;
                                let restore_id = backup_id.clone();
                                let delete_id = backup_id.clone();
                                let local_time = format_local_time(&backup.created_at.to_rfc3339(), locale.get_untracked());
                                view! {
                                    <tr>
                                        <td>
                                            <div class="stacked-cell">
                                                <strong>{local_time}</strong>
                                                <span>{backup_id}</span>
                                            </div>
                                        </td>
                                        <td>{move || format!("{} {}", size, t(locale.get(), Msg::BackupSize))}</td>
                                        <td>{path}</td>
                                        <td class="actions-cell">
                                            <div class="row-actions">
                                                <Button
                                                    size=ButtonSize::Small
                                                    variant=ButtonVariant::Subtle
                                                    on_click=move |_| on_restore.run(restore_id.clone())
                                                >
                                                    {move || t(locale.get(), Msg::Restore)}
                                                </Button>
                                                <Button
                                                    size=ButtonSize::Small
                                                    variant=ButtonVariant::Subtle
                                                    on_click=move |_| on_delete.run(delete_id.clone())
                                                >
                                                    {move || t(locale.get(), Msg::Delete)}
                                                </Button>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </section>
    }
}

fn format_local_time(value: &str, locale: Locale) -> String {
    #[cfg(feature = "hydrate")]
    {
        format_browser_local_time(value, locale)
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = locale;
        value.into()
    }
}

#[cfg(feature = "hydrate")]
fn format_browser_local_time(value: &str, locale: Locale) -> String {
    let date = js_sys::Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.into();
    }
    date.to_locale_string(locale.code(), &JsValue::UNDEFINED)
        .as_string()
        .unwrap_or_else(|| value.into())
}
