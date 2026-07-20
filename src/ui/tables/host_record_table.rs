use leptos::prelude::*;

use crate::config::model::HostRecord;
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::editable_table::{
    EditableTable, EditableTableActions, EditableTableColumns, RecordEditor,
};
use crate::ui::components::form_controls::{Field, Input};
use crate::ui::tables::{EditableRow, find_row, remove_row, upsert_row};
use crate::ui::text::localized;

#[component]
pub fn host_record_table(
    records: RwSignal<Vec<EditableRow<HostRecord>>>,
    locale: Signal<Locale>,
) -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<u64>);
    let names = RwSignal::new(String::new());
    let ips = RwSignal::new(String::new());

    let open_new = move || {
        editing_id.set(None);
        names.set(String::new());
        ips.set(String::new());
        modal_open.set(true);
    };

    let open_edit = move |id: u64| {
        if let Some(value) = find_row(records, id) {
            value.with(|record| {
                names.set(record.names.join(", "));
                ips.set(record.ips.join(", "));
            });
            editing_id.set(Some(id));
            modal_open.set(true);
        }
    };

    let save = move || {
        upsert_row(
            records,
            editing_id.get_untracked(),
            HostRecord {
                names: split_csv(&names.get_untracked()),
                ips: split_csv(&ips.get_untracked()),
            },
        );
        modal_open.set(false);
    };

    view! {
        <EditableTable
            title=localized(locale, Msg::HostRecord)
            is_empty=Signal::derive(move || records.with(Vec::is_empty))
            empty_message=Signal::derive(move || t(locale.get(), Msg::HostRecordEmpty))
            locale=locale
            on_add=move |_| open_new()
        >
            <EditableTableColumns slot>
                <th scope="col">{move || t(locale.get(), Msg::Name)}</th>
                <th scope="col">{move || t(locale.get(), Msg::Ip)}</th>
            </EditableTableColumns>
            <For
                each=move || records.get()
                key=|row| row.id
                children=move |row| {
                    let id = row.id;
                    let value = row.value;
                    view! {
                        <tr>
                            <td>{move || value.with(|record| record.names.join(", "))}</td>
                            <td>{move || value.with(|record| record.ips.join(", "))}</td>
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
            title=localized(locale, Msg::HostRecord)
            locale=locale
            on_save=move |_| save()
        >
            <Field label=localized(locale, Msg::Name)>
                <Input
                    value=names
                    placeholder=localized(locale, Msg::HostRecordNamesPlaceholder)
                />
            </Field>
            <Field label=localized(locale, Msg::Ip)>
                <Input
                    value=ips
                    placeholder=localized(locale, Msg::HostRecordIpsPlaceholder)
                />
            </Field>
        </RecordEditor>
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
