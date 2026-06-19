use leptos::prelude::*;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Field, Input, Table, TableBody, TableCell, TableHeader, TableHeaderCell, TableRow,
};

use crate::config::model::HostRecord;
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::editable_table::EditableTable;
use crate::ui::tables::{EditableRow, find_row, remove_row, upsert_row};
use crate::ui::text::localized;

#[component]
pub fn HostRecordTable(
    records: RwSignal<Vec<EditableRow<HostRecord>>>,
    locale: Signal<Locale>,
) -> impl IntoView {
    let dialog_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<u64>);
    let names = RwSignal::new(String::new());
    let ips = RwSignal::new(String::new());

    let open_new = move || {
        editing_id.set(None);
        names.set(String::new());
        ips.set(String::new());
        dialog_open.set(true);
    };

    let open_edit = move |id: u64| {
        if let Some(value) = find_row(records, id) {
            value.with(|record| {
                names.set(record.names.join(", "));
                ips.set(record.ips.join(", "));
            });
            editing_id.set(Some(id));
            dialog_open.set(true);
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
        dialog_open.set(false);
    };

    view! {
        <section class="table-section">
            <div class="section-head">
                <h2>{move || t(locale.get(), Msg::HostRecord)}</h2>
                <Button button_type=thaw::ButtonType::Button on_click=move |_| open_new()>
                    {move || t(locale.get(), Msg::Add)}
                </Button>
            </div>

            <EditableTable
                is_empty=Signal::derive(move || records.with(Vec::is_empty))
                empty_message=Signal::derive(move || t(locale.get(), Msg::HostRecordEmpty))
            >
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHeaderCell>{move || t(locale.get(), Msg::Name)}</TableHeaderCell>
                            <TableHeaderCell>{move || t(locale.get(), Msg::Ip)}</TableHeaderCell>
                            <TableHeaderCell class="actions-col">{move || t(locale.get(), Msg::Actions)}</TableHeaderCell>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <For
                            each=move || records.get()
                            key=|row| row.id
                            children=move |row| {
                                let id = row.id;
                                let value = row.value;
                                view! {
                                    <TableRow>
                                        <TableCell>{move || value.with(|record| record.names.join(", "))}</TableCell>
                                        <TableCell>{move || value.with(|record| record.ips.join(", "))}</TableCell>
                                        <TableCell class="actions-cell">
                                            <div class="row-actions">
                                                <Button
                                                    size=thaw::ButtonSize::Small
                                                    button_type=thaw::ButtonType::Button
                                                    on_click=move |_| open_edit(id)
                                                >
                                                    {move || t(locale.get(), Msg::Edit)}
                                                </Button>
                                                <Button
                                                    size=thaw::ButtonSize::Small
                                                    appearance=ButtonAppearance::Subtle
                                                    button_type=thaw::ButtonType::Button
                                                    on_click=move |_| remove_row(records, id)
                                                >
                                                    {move || t(locale.get(), Msg::Delete)}
                                                </Button>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                }
                            }
                        />
                    </TableBody>
                </Table>
            </EditableTable>

            <Dialog open=dialog_open>
                <DialogSurface>
                    <DialogContent>
                        <DialogTitle>{move || t(locale.get(), Msg::HostRecord)}</DialogTitle>
                        <DialogBody>
                            <div class="dialog-form">
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
                            </div>
                        </DialogBody>
                        <DialogActions>
                            <Button button_type=thaw::ButtonType::Button on_click=move |_| dialog_open.set(false)>
                                {move || t(locale.get(), Msg::Cancel)}
                            </Button>
                            <Button
                                appearance=ButtonAppearance::Primary
                                button_type=thaw::ButtonType::Button
                                on_click=move |_| save()
                            >
                                {move || t(locale.get(), Msg::Save)}
                            </Button>
                        </DialogActions>
                    </DialogContent>
                </DialogSurface>
            </Dialog>
        </section>
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
