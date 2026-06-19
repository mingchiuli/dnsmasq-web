use leptos::prelude::*;
use thaw::{Button, ButtonAppearance, ButtonType};

use crate::i18n::{Locale, Msg, t};

#[component]
pub fn Toolbar(
    title: &'static str,
    #[prop(into)] on_refresh: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
    #[prop(into)] on_apply: Callback<()>,
    #[prop(into)] on_logout: Callback<()>,
    busy: Signal<bool>,
    locale: Signal<Locale>,
    #[prop(into)] on_toggle_locale: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="toolbar">
            <div>
                <h1>{title}</h1>
            </div>
            <div class="toolbar-actions">
                <Button button_type=ButtonType::Button on_click=move |_| on_toggle_locale.run(())>
                    {move || t(locale.get(), Msg::LocaleSwitch)}
                </Button>
                <Button button_type=ButtonType::Button on_click=move |_| on_refresh.run(()) disabled=busy>
                    {move || t(locale.get(), Msg::Refresh)}
                </Button>
                <Button button_type=ButtonType::Button on_click=move |_| on_save.run(()) disabled=busy>
                    {move || t(locale.get(), Msg::Save)}
                </Button>
                <Button
                    appearance=ButtonAppearance::Primary
                    button_type=ButtonType::Button
                    on_click=move |_| on_apply.run(())
                    disabled=busy
                >
                    {move || t(locale.get(), Msg::Apply)}
                </Button>
                <Button button_type=ButtonType::Button on_click=move |_| on_logout.run(())>
                    {move || t(locale.get(), Msg::Logout)}
                </Button>
            </div>
        </div>
    }
}
