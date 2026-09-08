use leptos::prelude::*;

use crate::config::model::{AddressRecord, ServerRecord};
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::editable_table::{
    EditableTable, EditableTableActions, EditableTableColumns, RecordEditor,
};
use crate::ui::components::form_controls::{Field, Input};
use crate::ui::tables::address_rules::{has_local_rule, save_address};
use crate::ui::tables::{EditableRow, find_row, remove_row};
use crate::ui::text::localized;

#[component]
pub fn address_table(
    records: RwSignal<Vec<EditableRow<AddressRecord>>>,
    servers: RwSignal<Vec<EditableRow<ServerRecord>>>,
    locale: Signal<Locale>,
) -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<u64>);
    let domain = RwSignal::new(String::new());
    let ip = RwSignal::new(String::new());
    let local_choice = RwSignal::new(None::<bool>);
    let invalid = RwSignal::new(false);
    let local_only = Memo::new(move |_| {
        local_choice
            .get()
            .unwrap_or_else(|| servers.with(|rows| has_local_rule(rows, &domain.get())))
    });

    let open_new = move || {
        editing_id.set(None);
        domain.set(String::new());
        ip.set(String::new());
        local_choice.set(None);
        invalid.set(false);
        modal_open.set(true);
    };

    let open_edit = move |id: u64| {
        if let Some(value) = find_row(records, id) {
            value.with(|record| {
                domain.set(record.domain.clone());
                ip.set(record.ip.clone());
            });
            editing_id.set(Some(id));
            local_choice.set(None);
            invalid.set(false);
            modal_open.set(true);
        }
    };

    let save = move || {
        if !save_address(
            records,
            servers,
            editing_id.get_untracked(),
            AddressRecord {
                domain: domain.get_untracked().trim().into(),
                ip: ip.get_untracked().trim().into(),
            },
            local_only.get_untracked(),
        ) {
            invalid.set(true);
            return;
        }
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
                <th scope="col">{move || t(locale.get(), Msg::RecordType)}</th>
                <th scope="col">{move || t(locale.get(), Msg::Ip)}</th>
                <th scope="col">{move || t(locale.get(), Msg::LocalOnly)}</th>
            </EditableTableColumns>
            <For
                each=move || records.get()
                key=|row| row.id
                children=move |row| {
                    let id = row.id;
                    let value = RwSignal::from(row.value);
                    view! {
                        <tr>
                            <td>{move || value.with(|record| record.domain.clone())}</td>
                            <td>{move || value.with(|record| match record.ip.parse::<std::net::IpAddr>() {
                                Ok(std::net::IpAddr::V4(_)) => "A",
                                Ok(std::net::IpAddr::V6(_)) => "AAAA",
                                Err(_) => "—",
                            })}</td>
                            <td>{move || value.with(|record| record.ip.clone())}</td>
                            <td>{move || value.with(|record| servers.with(|rows| {
                                t(locale.get(), if has_local_rule(rows, &record.domain) {
                                    Msg::LocalRuleEnabled
                                } else {
                                    Msg::LocalRuleNotSet
                                })
                            }))}</td>
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
        <p class="muted">{move || t(locale.get(), Msg::AddressHelp)}</p>

        <RecordEditor
            open=modal_open
            title=localized(locale, Msg::Address)
            locale=locale
            on_save=move |_| save()
        >
            <Field label=localized(locale, Msg::Domain)>
                <input
                    class="ui-input"
                    type="text"
                    value=move || domain.get()
                    prop:value=move || domain.get()
                    placeholder=move || t(locale.get(), Msg::AddressDomainPlaceholder)
                    on:input=move |event| {
                        domain.set(event_target_value(&event));
                        local_choice.set(None);
                        invalid.set(false);
                    }
                />
            </Field>
            <Field label=localized(locale, Msg::Ip)>
                <Input value=ip placeholder=localized(locale, Msg::AddressIpPlaceholder) />
            </Field>
            <label class="local-only-option">
                <input
                    type="checkbox"
                    checked=move || local_only.get()
                    prop:checked=move || local_only.get()
                    on:change=move |event| local_choice.set(Some(event_target_checked(&event)))
                />
                <span>{move || t(locale.get(), Msg::AddressLocalOnly)}</span>
            </label>
            <p class="muted">{move || t(locale.get(), Msg::AddressLocalHelp)}</p>
            <Show when=move || invalid.get()>
                <p role="alert">{move || t(locale.get(), Msg::AddressInvalid)}</p>
            </Show>
        </RecordEditor>
    }
}
