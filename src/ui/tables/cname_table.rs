use leptos::prelude::*;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Field, Input, Table, TableBody, TableCell, TableHeader, TableHeaderCell, TableRow,
};

use crate::config::model::CnameRecord;
use crate::i18n::{Locale, Msg, t};
use crate::ui::components::editable_table::EditableTable;
use crate::ui::tables::{EditableRecord, remove_record, upsert_record};
use crate::ui::text::localized;

#[component]
pub fn CnameTable(
    records: RwSignal<Vec<EditableRecord<CnameRecord>>>,
    locale: Signal<Locale>,
) -> impl IntoView {
    let dialog_open = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<u64>);
    let alias = RwSignal::new(String::new());
    let canonical = RwSignal::new(String::new());

    let open_new = move || {
        editing_id.set(None);
        alias.set(String::new());
        canonical.set(String::new());
        dialog_open.set(true);
    };

    let open_edit = move |row: EditableRecord<CnameRecord>| {
        editing_id.set(Some(row.id));
        alias.set(row.record.alias);
        canonical.set(row.record.canonical);
        dialog_open.set(true);
    };

    let save = move || {
        upsert_record(
            records,
            editing_id.get_untracked(),
            CnameRecord {
                alias: alias.get_untracked(),
                canonical: canonical.get_untracked(),
            },
        );
        dialog_open.set(false);
    };

    view! {
        <section class="table-section">
            <div class="section-head">
                <h2>{move || t(locale.get(), Msg::Cname)}</h2>
                <Button button_type=thaw::ButtonType::Button on_click=move |_| open_new()>
                    {move || t(locale.get(), Msg::Add)}
                </Button>
            </div>

            <EditableTable
                is_empty=Signal::derive(move || records.get().is_empty())
                empty_message=Signal::derive(move || t(locale.get(), Msg::CnameEmpty))
            >
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHeaderCell>{move || t(locale.get(), Msg::Alias)}</TableHeaderCell>
                            <TableHeaderCell>{move || t(locale.get(), Msg::Domain)}</TableHeaderCell>
                            <TableHeaderCell class="actions-col">{move || t(locale.get(), Msg::Actions)}</TableHeaderCell>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <For
                            each=move || records.get()
                            key=|row| row.id
                            children=move |row| {
                                let edit_row = row.clone();
                                let id = row.id;
                                view! {
                                    <TableRow>
                                        <TableCell>{row.record.alias}</TableCell>
                                        <TableCell>{row.record.canonical}</TableCell>
                                        <TableCell class="actions-cell">
                                            <div class="row-actions">
                                                <Button
                                                    size=thaw::ButtonSize::Small
                                                    button_type=thaw::ButtonType::Button
                                                    on_click=move |_| open_edit(edit_row.clone())
                                                >
                                                    {move || t(locale.get(), Msg::Edit)}
                                                </Button>
                                                <Button
                                                    size=thaw::ButtonSize::Small
                                                    appearance=ButtonAppearance::Subtle
                                                    button_type=thaw::ButtonType::Button
                                                    on_click=move |_| remove_record(records, id)
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
                        <DialogTitle>{move || t(locale.get(), Msg::Cname)}</DialogTitle>
                        <DialogBody>
                            <div class="dialog-form">
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
