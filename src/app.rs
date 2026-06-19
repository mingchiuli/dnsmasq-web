use leptos::prelude::*;
use thaw::ConfigProvider;

use crate::ui::pages::records::RecordsPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <ConfigProvider>
            <RecordsPage />
        </ConfigProvider>
    }
}
