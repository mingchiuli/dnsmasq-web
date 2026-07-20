use leptos::{ev, prelude::*};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    #[default]
    Secondary,
    Subtle,
}

impl ButtonVariant {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Subtle => "subtle",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
}

impl ButtonSize {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonType {
    #[default]
    Button,
    Submit,
}

impl ButtonType {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Submit => "submit",
        }
    }
}

#[component]
pub fn button(
    #[prop(default = ButtonVariant::Secondary)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Medium)] size: ButtonSize,
    #[prop(default = ButtonType::Button)] button_type: ButtonType,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] on_click: Option<Callback<ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let class = format!(
        "ui-button ui-button--{} ui-button--{}",
        variant.as_str(),
        size.as_str()
    );

    view! {
        <button
            class=class
            type=button_type.as_str()
            disabled=move || disabled.get()
            on:click=move |event| {
                if let Some(on_click) = on_click {
                    on_click.run(event);
                }
            }
        >
            {children()}
        </button>
    }
}
