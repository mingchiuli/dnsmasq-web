use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::i18n::{Locale, Msg, t};
use crate::ui::api;
use crate::ui::components::button::{Button, ButtonType, ButtonVariant};
use crate::ui::components::form_controls::{Field, Input, InputType};
use crate::ui::components::modal::{Modal, ModalActions};
use crate::ui::components::notice::{Notice, NoticeTone};
use crate::ui::text::localized;

use super::controller::DashboardController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AuthMode {
    Setup,
    Login,
    Authenticated,
}

#[derive(Clone, Copy)]
pub(super) struct AuthState {
    pub(super) mode: RwSignal<AuthMode>,
    pub(super) password: RwSignal<String>,
    pub(super) password_confirmation: RwSignal<String>,
    pub(super) change_password_open: RwSignal<bool>,
    pub(super) current_password: RwSignal<String>,
    pub(super) new_password: RwSignal<String>,
    pub(super) new_password_confirmation: RwSignal<String>,
    pub(super) change_password_error: RwSignal<Option<String>>,
}

impl AuthState {
    pub(super) fn new() -> Self {
        Self {
            mode: RwSignal::new(AuthMode::Login),
            password: RwSignal::new(String::new()),
            password_confirmation: RwSignal::new(String::new()),
            change_password_open: RwSignal::new(false),
            current_password: RwSignal::new(String::new()),
            new_password: RwSignal::new(String::new()),
            new_password_confirmation: RwSignal::new(String::new()),
            change_password_error: RwSignal::new(None),
        }
    }

    pub(super) fn clear_login_form(self) {
        self.password.set(String::new());
        self.password_confirmation.set(String::new());
    }

    fn clear_change_password_form(self) {
        self.current_password.set(String::new());
        self.new_password.set(String::new());
        self.new_password_confirmation.set(String::new());
        self.change_password_error.set(None);
    }
}

pub(super) fn is_unauthorized(error: &str) -> bool {
    error.contains("unauthorized")
}

impl DashboardController {
    pub(super) fn submit_auth(self) {
        self.notice.clear();
        let password = self.auth.password.get();
        let mode = self.auth.mode.get_untracked();
        let confirmation = self.auth.password_confirmation.get();
        if mode == AuthMode::Setup && password != confirmation {
            self.notice.show_localized(Msg::PasswordsDoNotMatch);
            return;
        }

        self.busy.set(true);
        spawn_local(async move {
            let response = match mode {
                AuthMode::Setup => api::setup_password(password, confirmation).await,
                AuthMode::Login => api::login(password).await,
                AuthMode::Authenticated => {
                    self.busy.set(false);
                    return;
                }
            };

            match response {
                Ok(_) => {
                    self.auth.clear_login_form();
                    self.load_all();
                }
                Err(error) => {
                    self.notice.show_raw(error);
                    self.busy.set(false);
                }
            }
        });
    }

    pub(super) fn logout(self) {
        self.auth.clear_login_form();
        self.clear_dashboard();
        self.auth.mode.set(AuthMode::Login);
        self.notice.clear();
        spawn_local(async move {
            let _ = api::logout().await;
        });
    }

    pub(super) fn open_change_password(self) {
        self.auth.clear_change_password_form();
        self.auth.change_password_open.set(true);
    }

    pub(super) fn cancel_change_password(self) {
        if self.busy.get_untracked() {
            return;
        }
        self.auth.change_password_open.set(false);
        self.auth.clear_change_password_form();
    }

    pub(super) fn change_password(self) {
        self.auth.change_password_error.set(None);
        let current = self.auth.current_password.get_untracked();
        let new = self.auth.new_password.get_untracked();
        let confirmation = self.auth.new_password_confirmation.get_untracked();
        if new != confirmation {
            self.auth.change_password_error.set(Some(
                t(self.locale.get_untracked(), Msg::PasswordsDoNotMatch).into(),
            ));
            return;
        }

        self.busy.set(true);
        spawn_local(async move {
            match api::change_password(current, new, confirmation).await {
                Ok(_) => {
                    self.auth.change_password_open.set(false);
                    self.auth.clear_change_password_form();
                    self.notice.show_localized(Msg::PasswordChanged);
                }
                Err(error) if is_unauthorized(&error) => {
                    self.auth.change_password_open.set(false);
                    self.auth.clear_change_password_form();
                    self.handle_error(error);
                }
                Err(error) => self.auth.change_password_error.set(Some(error)),
            }
            self.busy.set(false);
        });
    }
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
                    AuthMode::Authenticated => t(locale.get(), Msg::Login),
                }}</h2>
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
                        AuthMode::Login | AuthMode::Authenticated => t(locale.get(), Msg::Login),
                    }}
                </Button>
                <Show when=move || message_visible.get()>
                    <Notice tone=NoticeTone::Error multiline=true>
                        {move || message_text.get()}
                    </Notice>
                </Show>
            </form>
        </div>
    }
}

#[component]
pub(super) fn change_password_dialog(
    open: RwSignal<bool>,
    current_password: RwSignal<String>,
    new_password: RwSignal<String>,
    new_password_confirmation: RwSignal<String>,
    busy: Signal<bool>,
    error: Signal<Option<String>>,
    locale: Signal<Locale>,
    #[prop(into)] on_save: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <Modal
            open=open
            title=localized(locale, Msg::ChangePassword)
            on_dismiss=move |_| on_cancel.run(())
        >
            <div class="modal-form">
                <Field label=localized(locale, Msg::CurrentPassword)>
                    <Input
                        value=current_password
                        input_type=InputType::Password
                        autocomplete="current-password"
                    />
                </Field>
                <Field label=localized(locale, Msg::NewPassword)>
                    <Input
                        value=new_password
                        input_type=InputType::Password
                        autocomplete="new-password"
                    />
                </Field>
                <Field label=localized(locale, Msg::ConfirmPassword)>
                    <Input
                        value=new_password_confirmation
                        input_type=InputType::Password
                        autocomplete="new-password"
                    />
                </Field>
                <Show when=move || error.with(Option::is_some)>
                    <Notice tone=NoticeTone::Error multiline=true>
                        {move || error.get().unwrap_or_default()}
                    </Notice>
                </Show>
            </div>
            <ModalActions slot>
                <Button on_click=move |_| on_cancel.run(()) disabled=busy>
                    {move || t(locale.get(), Msg::Cancel)}
                </Button>
                <Button
                    variant=ButtonVariant::Primary
                    on_click=move |_| on_save.run(())
                    disabled=busy
                >
                    {move || t(locale.get(), Msg::Save)}
                </Button>
            </ModalActions>
        </Modal>
    }
}
