use leptos::prelude::*;
use thaw::{
    Button, ButtonAppearance, ButtonType, Table, TableBody, TableCell, TableHeader,
    TableHeaderCell, TableRow,
};
use wasm_bindgen::JsValue;

use crate::api_types::BackupInfo;
use crate::i18n::{Locale, Msg, t};

#[component]
pub fn BackupsPanel(
    backups: Signal<Vec<BackupInfo>>,
    #[prop(into)] on_refresh: Callback<()>,
    #[prop(into)] on_restore: Callback<String>,
    locale: Signal<Locale>,
) -> impl IntoView {
    view! {
        <section class="backups">
            <div class="section-head">
                <h2>{move || t(locale.get(), Msg::Backups)}</h2>
                <Button button_type=ButtonType::Button on_click=move |_| on_refresh.run(())>
                    {move || t(locale.get(), Msg::Refresh)}
                </Button>
            </div>
            <div class="record-table">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHeaderCell>{move || t(locale.get(), Msg::BackupId)}</TableHeaderCell>
                            <TableHeaderCell>{move || t(locale.get(), Msg::BackupSize)}</TableHeaderCell>
                            <TableHeaderCell>"Path"</TableHeaderCell>
                            <TableHeaderCell class="actions-col">{move || t(locale.get(), Msg::Actions)}</TableHeaderCell>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <For
                            each=move || backups.get()
                            key=|backup| backup.id.clone()
                            children=move |backup| {
                                let backup_id = backup.id.clone();
                                let path = backup.path.clone();
                                let size = backup.size;
                                let id = backup_id.clone();
                                let local_time = format_local_time(&backup.created_at.to_rfc3339(), locale.get_untracked());
                                view! {
                                    <TableRow>
                                        <TableCell>
                                            <div class="stacked-cell">
                                                <strong>{local_time}</strong>
                                                <span>{backup_id}</span>
                                            </div>
                                        </TableCell>
                                        <TableCell>{move || format!("{} {}", size, t(locale.get(), Msg::BackupSize))}</TableCell>
                                        <TableCell>{path}</TableCell>
                                        <TableCell class="actions-cell">
                                            <Button
                                                size=thaw::ButtonSize::Small
                                                appearance=ButtonAppearance::Subtle
                                                button_type=ButtonType::Button
                                                on_click=move |_| on_restore.run(id.clone())
                                            >
                                                {move || t(locale.get(), Msg::Restore)}
                                            </Button>
                                        </TableCell>
                                    </TableRow>
                                }
                            }
                        />
                    </TableBody>
                </Table>
            </div>
        </section>
    }
}

fn format_local_time(value: &str, locale: Locale) -> String {
    let date = js_sys::Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.into();
    }
    date.to_locale_string(locale.code(), &JsValue::UNDEFINED)
        .as_string()
        .unwrap_or_else(|| value.into())
}
