use leptos::prelude::*;

use crate::i18n::{Locale, Msg, t};
use crate::ui::components::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::components::modal::{Modal, ModalActions};

#[slot]
pub struct EditableTableColumns {
    children: ChildrenFn,
}

#[component]
pub fn editable_table(
    title: Signal<String>,
    is_empty: Signal<bool>,
    empty_message: Signal<&'static str>,
    locale: Signal<Locale>,
    #[prop(into)] on_add: Callback<()>,
    editable_table_columns: EditableTableColumns,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <section class="table-section">
            <div class="section-head">
                <h2>{move || title.get()}</h2>
                <Button on_click=move |_| on_add.run(())>
                    {move || t(locale.get(), Msg::Add)}
                </Button>
            </div>

            <Show
                when=move || !is_empty.get()
                fallback=move || view! {
                    <div class="empty-table">{move || empty_message.get()}</div>
                }
            >
                <div class="record-table">
                    <table class="ui-table">
                        <thead>
                            <tr>
                                {(editable_table_columns.children)()}
                                <th scope="col" class="actions-col">
                                    {move || t(locale.get(), Msg::Actions)}
                                </th>
                            </tr>
                        </thead>
                        <tbody>{children()}</tbody>
                    </table>
                </div>
            </Show>
        </section>
    }
    .into_any()
}

#[component]
pub fn editable_table_actions(
    locale: Signal<Locale>,
    #[prop(into)] on_edit: Callback<()>,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    view! {
        <td class="actions-cell">
            <div class="row-actions">
                <Button size=ButtonSize::Small on_click=move |_| on_edit.run(())>
                    {move || t(locale.get(), Msg::Edit)}
                </Button>
                <Button
                    size=ButtonSize::Small
                    variant=ButtonVariant::Subtle
                    on_click=move |_| on_delete.run(())
                >
                    {move || t(locale.get(), Msg::Delete)}
                </Button>
            </div>
        </td>
    }
}

#[component]
pub fn record_editor(
    open: RwSignal<bool>,
    title: Signal<String>,
    locale: Signal<Locale>,
    #[prop(into)] on_save: Callback<()>,
    children: Children,
) -> impl IntoView {
    view! {
        <Modal
            open=open
            title=title
            on_dismiss=move |_| open.set(false)
        >
            <div class="modal-form">{children()}</div>
            <ModalActions slot>
                <Button on_click=move |_| open.set(false)>
                    {move || t(locale.get(), Msg::Cancel)}
                </Button>
                <Button
                    variant=ButtonVariant::Primary
                    on_click=move |_| on_save.run(())
                >
                    {move || t(locale.get(), Msg::Save)}
                </Button>
            </ModalActions>
        </Modal>
    }
    .into_any()
}
