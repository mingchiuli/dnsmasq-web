use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::ui::pages::dashboard::DashboardPage;

#[component]
pub fn app() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=DashboardPage>
                <Route path=path!("") view=DashboardPage />
            </Routes>
        </Router>
    }
}

#[cfg(feature = "ssr")]
#[component]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    let stylesheet = options.css_path();

    view! {
        <!DOCTYPE html>
        <html lang="zh-CN">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>"dnsmasq-web"</title>
                <link rel="icon" href="/favicon.png" type="image/png" />
                <link rel="apple-touch-icon" href="/icon-192.png" />
                <link rel="manifest" href="/site.webmanifest" />
                <link rel="stylesheet" href=stylesheet />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
