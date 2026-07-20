use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputType {
    #[default]
    Text,
    Password,
}

impl InputType {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password => "password",
        }
    }
}

#[component]
pub fn field(label: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <label class="ui-field">
            <span class="ui-field__label">{move || label.get()}</span>
            {children()}
        </label>
    }
}

#[component]
pub fn input(
    value: RwSignal<String>,
    #[prop(default = InputType::Text)] input_type: InputType,
    #[prop(optional, into)] placeholder: MaybeProp<String>,
    #[prop(optional, into)] autocomplete: MaybeProp<String>,
) -> impl IntoView {
    view! {
        <input
            class="ui-input"
            type=input_type.as_str()
            value=move || value.get()
            prop:value=move || value.get()
            placeholder=move || placeholder.get()
            autocomplete=move || autocomplete.get()
            on:input=move |event| value.set(event_target_value(&event))
        />
    }
}

#[component]
pub fn textarea(
    value: RwSignal<String>,
    #[prop(optional, into)] class: MaybeProp<String>,
) -> impl IntoView {
    view! {
        <textarea
            class=move || match class.get() {
                Some(class) => format!("ui-textarea {class}"),
                None => String::from("ui-textarea"),
            }
            prop:value=move || value.get()
            on:input=move |event| value.set(event_target_value(&event))
        ></textarea>
    }
}
