use leptos::prelude::*;

use crate::i18n::{Locale, Msg, t};
use crate::ui::components::button::{Button, ButtonVariant};
use crate::ui::components::modal::{Modal, ModalActions};
use crate::ui::text::localized;

#[component]
pub fn confirm_dialog(
    open: RwSignal<bool>,
    message: Signal<String>,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
    locale: Signal<Locale>,
) -> impl IntoView {
    view! {
        <Modal
            open=open
            title=localized(locale, Msg::Confirm)
            on_dismiss=move |_| on_cancel.run(())
        >
            <p>{move || message.get()}</p>
            <ModalActions slot>
                <Button on_click=move |_| on_cancel.run(())>
                    {move || t(locale.get(), Msg::Cancel)}
                </Button>
                <Button
                    variant=ButtonVariant::Primary
                    on_click=move |_| on_confirm.run(())
                >
                    {move || t(locale.get(), Msg::Confirm)}
                </Button>
            </ModalActions>
        </Modal>
    }
}
