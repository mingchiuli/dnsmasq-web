use leptos::prelude::*;

use crate::config::model::ServerRecord;
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

    let open_new = move || {
        editing_id.set(None);
        domain.set(String::new());
        upstream.set(String::new());
        modal_open.set(true);
    };

    let open_edit = move |id: u64| {
        if let Some(value) = find_row(records, id) {
            value.with(|record| {
                domain.set(record.domain.clone().unwrap_or_default());
                upstream.set(record.upstream.clone());
            });
            editing_id.set(Some(id));
            modal_open.set(true);
        }
    };

    let save = move || {
        let domain_value = domain.get_untracked();
        upsert_row(
            records,
            editing_id.get_untracked(),
            ServerRecord {
                domain: non_empty(domain_value),
                upstream: upstream.get_untracked(),
            },
        );
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
                <th scope="col">{move || t(locale.get(), Msg::Upstream)}</th>
            </EditableTableColumns>
            <For
                each=move || records.get()
                key=|row| row.id
                children=move |row| {
                    let id = row.id;
                    let value = row.value;
                    view! {
                        <tr>
                            <td>
                                {move || value.with(|record| record.domain.as_deref().unwrap_or("*").to_string())}
                            </td>
                            <td>{move || value.with(|record| record.upstream.clone())}</td>
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

        <RecordEditor
            open=modal_open
            title=localized(locale, Msg::Server)
            locale=locale
            on_save=move |_| save()
        >
            <Field label=localized(locale, Msg::DomainScope)>
                <Input
                    value=domain
                    placeholder=localized(locale, Msg::ServerDomainPlaceholder)
                />
            </Field>
            <Field label=localized(locale, Msg::Upstream)>
                <Input
                    value=upstream
                    placeholder=localized(locale, Msg::ServerUpstreamPlaceholder)
                />
            </Field>
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
