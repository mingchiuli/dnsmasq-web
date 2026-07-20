use leptos::prelude::*;

use crate::i18n::{Locale, Msg, t};
use crate::ui::components::button::{Button, ButtonType, ButtonVariant};
use crate::ui::components::form_controls::{Field, Input, InputType};
use crate::ui::components::notice::{Notice, NoticeTone};
use crate::ui::text::localized;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AuthMode {
    Loading,
    Setup,
    Login,
    Authenticated,
}

pub(super) fn is_unauthorized(error: &str) -> bool {
    error.contains("unauthorized")
}

#[component]
pub(super) fn auth_gate(
    mode: Signal<AuthMode>,
    password: RwSignal<String>,
    password_confirmation: RwSignal<String>,
    busy: Signal<bool>,
    message_visible: Signal<bool>,
    message_text: Signal<String>,
    locale: Signal<Locale>,
    #[prop(into)] on_submit: Callback<()>,
    #[prop(into)] on_toggle_locale: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="auth-shell">
            <div class="auth-head">
                <h1>"dnsmasq-web"</h1>
                <Button on_click=move |_| on_toggle_locale.run(())>
                    {move || t(locale.get(), Msg::LocaleSwitch)}
                </Button>
            </div>
            <form
                class="auth-panel"
                on:submit=move |ev| {
                    ev.prevent_default();
                    on_submit.run(());
                }
            >
                <h2>{move || match mode.get() {
                    AuthMode::Setup => t(locale.get(), Msg::SetupPassword),
                    AuthMode::Login => t(locale.get(), Msg::Login),
                    AuthMode::Loading | AuthMode::Authenticated => t(locale.get(), Msg::Loading),
                }}</h2>
                <Show when=move || mode.get() != AuthMode::Loading>
                    <Show
                        when=move || mode.get() == AuthMode::Setup
                        fallback=move || view! {
                            <Field label=localized(locale, Msg::Password)>
                                <Input
                                    value=password
                                    input_type=InputType::Password
                                    autocomplete="current-password"
                                />
                            </Field>
                        }
                    >
                        <Field label=localized(locale, Msg::Password)>
                            <Input
                                value=password
                                input_type=InputType::Password
                                autocomplete="new-password"
                            />
                        </Field>
                        <Field label=localized(locale, Msg::ConfirmPassword)>
                            <Input
                                value=password_confirmation
                                input_type=InputType::Password
                                autocomplete="new-password"
                            />
                        </Field>
                    </Show>
                    <Button
                        variant=ButtonVariant::Primary
                        button_type=ButtonType::Submit
                        disabled=busy
                    >
                        {move || match mode.get() {
                            AuthMode::Setup => t(locale.get(), Msg::SetPassword),
                            AuthMode::Login => t(locale.get(), Msg::Login),
                            AuthMode::Loading | AuthMode::Authenticated => t(locale.get(), Msg::Loading),
                        }}
                    </Button>
                </Show>
                <Show when=move || message_visible.get()>
                    <Notice tone=NoticeTone::Error multiline=true>
                        {move || message_text.get()}
                    </Notice>
                </Show>
            </form>
        </div>
    }
}
