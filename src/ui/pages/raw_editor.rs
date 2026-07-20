use leptos::prelude::*;

use crate::i18n::{Locale, Msg, t};
use crate::ui::components::button::Button;
use crate::ui::components::form_controls::Textarea;

#[component]
pub fn raw_editor_panel(
    content: RwSignal<String>,
    #[prop(into)] on_test: Callback<()>,
    locale: Signal<Locale>,
) -> impl IntoView {
    view! {
        <section class="raw-editor">
            <div class="section-head">
                <h2>{move || t(locale.get(), Msg::RawConfig)}</h2>
                <Button on_click=move |_| on_test.run(())>
                    {move || t(locale.get(), Msg::TestConfig)}
                </Button>
            </div>
            <Textarea class="raw-textarea" value=content />
        </section>
    }
}
