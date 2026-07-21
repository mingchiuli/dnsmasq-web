use leptos::prelude::*;

use crate::i18n::{Locale, Msg, t};

pub(super) enum NoticeMessage {
    Localized(Msg),
    LocalizedDetail { msg: Msg, detail: String },
    Raw(String),
}

#[derive(Clone, Copy)]
pub(super) struct NoticeState {
    message: RwSignal<Option<NoticeMessage>>,
}

impl NoticeState {
    pub(super) fn new() -> Self {
        Self {
            message: RwSignal::new(None),
        }
    }

    pub(super) fn show(self, message: NoticeMessage) {
        self.message.set(Some(message));
    }

    pub(super) fn show_localized(self, message: Msg) {
        self.show(NoticeMessage::Localized(message));
    }

    pub(super) fn show_raw(self, message: String) {
        self.show(NoticeMessage::Raw(message));
    }

    pub(super) fn clear(self) {
        self.message.set(None);
    }

    pub(super) fn visible(self) -> Signal<bool> {
        Signal::derive(move || self.message.with(Option::is_some))
    }

    pub(super) fn text(self, locale: RwSignal<Locale>) -> Signal<String> {
        Signal::derive(move || {
            let locale = locale.get();
            self.message.with(|message| {
                message
                    .as_ref()
                    .map(|message| message.render(locale))
                    .unwrap_or_default()
            })
        })
    }
}

impl NoticeMessage {
    pub(super) fn render(&self, locale: Locale) -> String {
        match self {
            Self::Localized(msg) => t(locale, *msg).into(),
            Self::LocalizedDetail { msg, detail } if detail.is_empty() => t(locale, *msg).into(),
            Self::LocalizedDetail { msg, detail } => {
                format!("{}: {}", t(locale, *msg), detail)
            }
            Self::Raw(message) => message.clone(),
        }
    }
}
