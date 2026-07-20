use leptos::prelude::*;

use crate::config::model::CnameRecord;
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::editable_table::{
    EditableTable, EditableTableActions, EditableTableColumns, RecordEditor,
};
use crate::ui::components::form_controls::{Field, Input};
use crate::ui::tables::{EditableRow, find_row, remove_row, upsert_row};
use crate::ui::text::localized;

#[component]
pub fn cname_table(
    records: RwSignal<Vec<EditableRow<CnameRecord>>>,
    locale: Signal<Locale>,
) -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<u64>);
    let alias = RwSignal::new(String::new());
    let canonical = RwSignal::new(String::new());

    let open_new = move || {
        editing_id.set(None);
        alias.set(String::new());
        canonical.set(String::new());
        modal_open.set(true);
    };

    let open_edit = move |id: u64| {
        if let Some(value) = find_row(records, id) {
            value.with(|record| {
                alias.set(record.alias.clone());
                canonical.set(record.canonical.clone());
            });
            editing_id.set(Some(id));
            modal_open.set(true);
        }
    };

    let save = move || {
        upsert_row(
            records,
            editing_id.get_untracked(),
            CnameRecord {
                alias: alias.get_untracked(),
                canonical: canonical.get_untracked(),
            },
        );
        modal_open.set(false);
    };

    view! {
        <EditableTable
            title=localized(locale, Msg::Cname)
            is_empty=Signal::derive(move || records.with(Vec::is_empty))
            empty_message=Signal::derive(move || t(locale.get(), Msg::CnameEmpty))
            locale=locale
            on_add=move |_| open_new()
        >
            <EditableTableColumns slot>
                <th scope="col">{move || t(locale.get(), Msg::Alias)}</th>
                <th scope="col">{move || t(locale.get(), Msg::Domain)}</th>
            </EditableTableColumns>
            <For
                each=move || records.get()
                key=|row| row.id
                children=move |row| {
                    let id = row.id;
                    let value = row.value;
                    view! {
                        <tr>
                            <td>{move || value.with(|record| record.alias.clone())}</td>
                            <td>{move || value.with(|record| record.canonical.clone())}</td>
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
            title=localized(locale, Msg::Cname)
            locale=locale
            on_save=move |_| save()
        >
            <Field label=localized(locale, Msg::Alias)>
                <Input
                    value=alias
                    placeholder=localized(locale, Msg::CnameAliasPlaceholder)
                />
            </Field>
            <Field label=localized(locale, Msg::Domain)>
                <Input
                    value=canonical
                    placeholder=localized(locale, Msg::CnameCanonicalPlaceholder)
                />
            </Field>
        </RecordEditor>
    }
}
