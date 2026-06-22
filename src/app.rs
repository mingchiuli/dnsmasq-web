use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use thaw::ConfigProvider;
#[cfg(feature = "ssr")]
use thaw::ssr::SSRMountStyleProvider;

use crate::ui::pages::dashboard::DashboardPage;

#[component]
pub fn app() -> impl IntoView {
    view! {
        <ConfigProvider>
            <Router>
                <Routes fallback=DashboardPage>
                    <Route path=path!("") view=DashboardPage />
                </Routes>
            </Router>
        </ConfigProvider>
    }
}

#[cfg(feature = "ssr")]
#[component]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    let stylesheet = options.css_path();

    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="zh-CN">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <meta name="thaw-ui-style" />
                    <title>"dnsmasq-web"</title>
                    <link rel="icon" href="/favicon.png" type="image/png" />
                    <link rel="apple-touch-icon" href="/icon-192.png" />
                    <link rel="manifest" href="/site.webmanifest" />
                    <AutoReload options=options.clone() />
                    <HydrationScripts options=options />
                    <link rel="stylesheet" href=stylesheet />
                </head>
                <body>
                    <App />
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}
