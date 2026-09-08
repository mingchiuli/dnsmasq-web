use leptos::prelude::*;

use crate::config::model::{DnsRecords, ServerRecord};
use crate::config::validate::{has_errors, validate_records};
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::editable_table::{
    EditableTable, EditableTableActions, EditableTableColumns, RecordEditor,
};
use crate::ui::components::form_controls::{Field, Input};
use crate::ui::tables::{EditableRow, find_row, remove_row, upsert_row};
use crate::ui::text::localized;

#[component]
pub fn server_table(
    records: RwSignal<Vec<EditableRow<ServerRecord>>>,
    locale: Signal<Locale>,
) -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<u64>);
    let domain = RwSignal::new(String::new());
    let upstream = RwSignal::new(String::new());
    let local_only = RwSignal::new(false);
    let invalid = RwSignal::new(false);

    let open_new = move || {
        editing_id.set(None);
        domain.set(String::new());
        upstream.set(String::new());
        local_only.set(false);
        invalid.set(false);
        modal_open.set(true);
    };

    let open_edit = move |id: u64| {
        if let Some(value) = find_row(records, id) {
            value.with(|record| {
                domain.set(record.domain.clone().unwrap_or_default());
                upstream.set(record.upstream.clone());
                local_only.set(record.upstream.trim().is_empty());
            });
            invalid.set(false);
            editing_id.set(Some(id));
            modal_open.set(true);
        }
    };

    let save = move || {
        let domain_value = domain.get_untracked();
        let record = ServerRecord {
            domain: non_empty(domain_value),
            upstream: if local_only.get_untracked() {
                String::new()
            } else {
                upstream.get_untracked().trim().to_string()
            },
        };
        if (!local_only.get_untracked() && record.upstream.is_empty())
            || has_errors(&validate_records(&DnsRecords {
                server: vec![record.clone()],
                ..DnsRecords::default()
            }))
        {
            invalid.set(true);
            return;
        }
        upsert_row(records, editing_id.get_untracked(), record);
        modal_open.set(false);
    };

    view! {
        <EditableTable
            title=localized(locale, Msg::Server)
            is_empty=Signal::derive(move || records.with(Vec::is_empty))
            empty_message=Signal::derive(move || t(locale.get(), Msg::ServerEmpty))
            locale=locale
            on_add=move |_| open_new()
        >
            <EditableTableColumns slot>
                <th scope="col">{move || t(locale.get(), Msg::DomainScope)}</th>
                <th scope="col">{move || t(locale.get(), Msg::ResolutionMode)}</th>
                <th scope="col">{move || t(locale.get(), Msg::Upstream)}</th>
            </EditableTableColumns>
            <For
                each=move || records.get()
                key=|row| row.id
                children=move |row| {
                    let id = row.id;
                    let value = RwSignal::from(row.value);
                    view! {
                        <tr>
                            <td>
                                {move || value.with(|record| record.domain.as_deref().unwrap_or("*").to_string())}
                            </td>
                            <td>{move || value.with(|record| t(locale.get(), if record.upstream.trim().is_empty() {
                                Msg::LocalOnly
                            } else {
                                Msg::ForwardUpstream
                            }))}</td>
                            <td>{move || value.with(|record| if record.upstream.trim().is_empty() {
                                String::from("—")
                            } else {
                                record.upstream.clone()
                            })}</td>
                            <EditableTableActions
                                locale=locale
                                on_edit=move |_| open_edit(id)
                                on_delete=move |_| remove_row(records, id)
                            />
                        </tr>
                    }
                }
            />
        </EditableTable>
        <p class="muted">{move || t(locale.get(), Msg::ServerLocalHelp)}</p>

        <RecordEditor
            open=modal_open
            title=localized(locale, Msg::Server)
            locale=locale
            on_save=move |_| save()
        >
            <Field label=localized(locale, Msg::ResolutionMode)>
                <select
                    class="ui-input"
                    prop:value=move || if local_only.get() { "local" } else { "forward" }
                    on:change=move |event| {
                        local_only.set(event_target_value(&event) == "local");
                        invalid.set(false);
                    }
                >
                    <option value="forward" selected=move || !local_only.get()>
                        {move || t(locale.get(), Msg::ForwardUpstream)}
                    </option>
                    <option value="local" selected=move || local_only.get()>
                        {move || t(locale.get(), Msg::LocalOnly)}
                    </option>
                </select>
            </Field>
            <Field label=localized(locale, Msg::DomainScope)>
                <Input
                    value=domain
                    placeholder=Signal::derive(move || t(locale.get(), if local_only.get() {
                        Msg::ServerLocalDomainPlaceholder
                    } else {
                        Msg::ServerDomainPlaceholder
                    }).to_string())
                />
            </Field>
            <Show when=move || !local_only.get()>
                <Field label=localized(locale, Msg::Upstream)>
                    <Input
                        value=upstream
                        placeholder=localized(locale, Msg::ServerUpstreamPlaceholder)
                    />
                </Field>
            </Show>
            <Show when=move || invalid.get()>
                <p role="alert">{move || t(locale.get(), if local_only.get() {
                    Msg::ServerLocalInvalid
                } else {
                    Msg::ServerForwardInvalid
                })}</p>
            </Show>
        </RecordEditor>
    }
}

fn non_empty(mut value: String) -> Option<String> {
    let trimmed_len = value.trim_end().len();
    value.truncate(trimmed_len);
    let trimmed_start = value.len() - value.trim_start().len();
    value.drain(..trimmed_start);

    if value.is_empty() { None } else { Some(value) }
}
