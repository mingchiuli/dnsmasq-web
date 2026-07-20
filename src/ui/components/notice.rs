use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoticeTone {
    #[default]
    Info,
    Warning,
    Error,
}

impl NoticeTone {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    const fn role(self) -> &'static str {
        match self {
            Self::Error => "alert",
            Self::Info | Self::Warning => "status",
        }
    }
}

#[component]
pub fn notice(
    #[prop(optional, into)] tone: Signal<NoticeTone>,
    #[prop(optional)] multiline: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=move || format!(
                "ui-notice ui-notice--{}{}",
                tone.get().as_str(),
                if multiline { " ui-notice--multiline" } else { "" }
            )
            role=move || tone.get().role()
        >
            <div class="ui-notice__body">{children()}</div>
        </div>
    }
    .into_any()
}
