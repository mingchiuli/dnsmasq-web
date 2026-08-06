use chrono::{DateTime, Utc};
use leptos::prelude::*;

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
                                let created_at = format_utc_time(&backup.created_at);
                                let backup_id = backup.id;
                                let path = backup.path;
                                let size = backup.size;
                                let restore_id = backup_id.clone();
                                let delete_id = backup_id.clone();
                                view! {
                                    <tr>
                                        <td>
                                            <div class="stacked-cell">
                                                <strong>{created_at}</strong>
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

fn format_utc_time(value: &DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::format_utc_time;

    #[test]
    fn formats_backup_time_as_utc() {
        let created_at = DateTime::parse_from_rfc3339("2026-08-06T20:34:56+08:00")
            .expect("valid timestamp")
            .with_timezone(&Utc);

        assert_eq!(format_utc_time(&created_at), "2026-08-06 12:34:56 UTC");
    }
}
