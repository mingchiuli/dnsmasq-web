use leptos::prelude::*;
use thaw::{
    Button, ButtonAppearance, ButtonType, Dialog, DialogActions, DialogBody, DialogContent,
    DialogSurface,
};

use crate::i18n::{Locale, Msg, t};

#[component]
pub fn ConfirmDialog(
    open: RwSignal<bool>,
    message: Signal<String>,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
    locale: Signal<Locale>,
) -> impl IntoView {
    view! {
        <Dialog open=open>
            <DialogSurface>
                <DialogContent>
                    <DialogBody>
                        <p>{move || message.get()}</p>
                    </DialogBody>
                    <DialogActions>
                        <Button button_type=ButtonType::Button on_click=move |_| on_cancel.run(())>
                            {move || t(locale.get(), Msg::Cancel)}
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Primary
                            button_type=ButtonType::Button
                            on_click=move |_| on_confirm.run(())
                        >
                            {move || t(locale.get(), Msg::Confirm)}
                        </Button>
                    </DialogActions>
                </DialogContent>
            </DialogSurface>
        </Dialog>
    }
}
