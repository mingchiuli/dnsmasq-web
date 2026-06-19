use leptos::prelude::*;

use crate::i18n::{Locale, Msg, t};

pub fn localized(locale: Signal<Locale>, msg: Msg) -> Signal<String> {
    Signal::derive(move || t(locale.get(), msg).into())
}
