use leptos::prelude::*;
use thaw::{Button, ButtonType, Textarea, TextareaResize};

use crate::i18n::{Locale, Msg, t};

#[component]
pub fn RawEditorPanel(
    content: RwSignal<String>,
    #[prop(into)] on_test: Callback<()>,
    locale: Signal<Locale>,
) -> impl IntoView {
    view! {
        <section class="raw-editor">
            <div class="section-head">
                <h2>{move || t(locale.get(), Msg::RawConfig)}</h2>
                <Button button_type=ButtonType::Button on_click=move |_| on_test.run(())>
                    {move || t(locale.get(), Msg::TestConfig)}
                </Button>
            </div>
            <Textarea
                class="raw-textarea"
                value=content
                resize=TextareaResize::Vertical
            />
        </section>
    }
}
