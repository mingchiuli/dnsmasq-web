use leptos::prelude::*;

use crate::config::model::AddressRecord;
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::editable_table::{
    EditableTable, EditableTableActions, EditableTableColumns, RecordEditor,
};
use crate::ui::components::form_controls::{Field, Input};
use crate::ui::tables::{EditableRow, find_row, remove_row, upsert_row};
use crate::ui::text::localized;

#[component]
pub fn address_table(
    records: RwSignal<Vec<EditableRow<AddressRecord>>>,
    locale: Signal<Locale>,
) -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<u64>);
    let domain = RwSignal::new(String::new());
    let ip = RwSignal::new(String::new());

    let open_new = move || {
        editing_id.set(None);
        domain.set(String::new());
        ip.set(String::new());
        modal_open.set(true);
    };

    let open_edit = move |id: u64| {
        if let Some(value) = find_row(records, id) {
            value.with(|record| {
                domain.set(record.domain.clone());
                ip.set(record.ip.clone());
            });
            editing_id.set(Some(id));
            modal_open.set(true);
        }
    };

    let save = move || {
        upsert_row(
            records,
            editing_id.get_untracked(),
            AddressRecord {
                domain: domain.get_untracked(),
                ip: ip.get_untracked(),
            },
        );
        modal_open.set(false);
    };

    view! {
        <EditableTable
            title=localized(locale, Msg::Address)
            is_empty=Signal::derive(move || records.with(Vec::is_empty))
            empty_message=Signal::derive(move || t(locale.get(), Msg::AddressEmpty))
            locale=locale
            on_add=move |_| open_new()
        >
            <EditableTableColumns slot>
                <th scope="col">{move || t(locale.get(), Msg::Domain)}</th>
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
                            <td>{move || value.with(|record| record.domain.clone())}</td>
                            <td>{move || value.with(|record| record.ip.clone())}</td>
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
            title=localized(locale, Msg::Address)
            locale=locale
            on_save=move |_| save()
        >
            <Field label=localized(locale, Msg::Domain)>
                <Input
                    value=domain
                    placeholder=localized(locale, Msg::AddressDomainPlaceholder)
                />
            </Field>
            <Field label=localized(locale, Msg::Ip)>
                <Input value=ip placeholder="10.10.0.1" />
            </Field>
        </RecordEditor>
    }
}
