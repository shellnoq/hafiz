//! Main application component with routing

use leptos::spawn_local;
use leptos::*;
use leptos_router::{use_navigate, Outlet, Route, Router, Routes, A};

use crate::components::{Header, Sidebar};
use crate::pages::{
    BucketDetailPage, BucketsPage, ClusterPage, DashboardPage, LdapSettingsPage, NotFoundPage,
    ObjectsPage, SettingsPage, UsersPage,
};

/// Root application component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="min-h-screen bg-gray-900 text-gray-100">
                <Routes>
                    <Route path="/login" view=LoginPage />
                    <Route path="/" view=MainLayout>
                        <Route path="" view=DashboardPage />
                        <Route path="buckets" view=BucketsPage />
                        <Route path="buckets/:name" view=BucketDetailPage />
                        <Route path="buckets/:name/objects/*path" view=ObjectsPage />
                        <Route path="users" view=UsersPage />
                        <Route path="cluster" view=ClusterPage />
                        <Route path="settings" view=SettingsPage />
                        <Route path="settings/ldap" view=LdapSettingsPage />
                        <Route path="/*any" view=NotFoundPage />
                    </Route>
                </Routes>
            </div>
        </Router>
    }
}

/// Main layout with sidebar and header
#[component]
fn MainLayout() -> impl IntoView {
    view! {
        <div class="flex h-screen">
            <Sidebar />
            <div class="flex-1 flex flex-col overflow-hidden">
                <Header />
                <main class="flex-1 overflow-y-auto p-6 bg-gray-800">
                    <Outlet />
                </main>
            </div>
        </div>
    }
}

/// Login page
#[component]
fn LoginPage() -> impl IntoView {
    let (access_key, set_access_key) = create_signal(String::new());
    let (secret_key, set_secret_key) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let ak = access_key.get();
        let sk = secret_key.get();

        // Validate credentials against API
        spawn_local(async move {
            match crate::api::validate_credentials(&ak, &sk).await {
                Ok(true) => {
                    // Credentials valid, redirect to dashboard
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Ok(false) => {
                    set_error.set(Some("Invalid credentials".to_string()));
                    set_loading.set(false);
                }
                Err(e) => {
                    // Network error - store credentials anyway for offline/development
                    log::warn!("Could not validate credentials: {}", e);
                    if let Some(storage) = web_sys::window()
                        .and_then(|w| w.local_storage().ok())
                        .flatten()
                    {
                        let _ = storage.set_item("hafiz_access_key", &ak);
                        let _ = storage.set_item("hafiz_secret_key", &sk);
                    }
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
            }
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-900">
            <div class="max-w-md w-full bg-gray-800 rounded-xl shadow-2xl p-8">
                <div class="text-center mb-8">
                    <div class="flex justify-center mb-4">
                        <svg class="w-16 h-16 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                        </svg>
                    </div>
                    <h1 class="text-3xl font-bold text-white">"Hafiz"</h1>
                    <p class="text-gray-400 mt-2">"S3-Compatible Object Storage"</p>
                </div>

                <form on:submit=on_submit class="space-y-6">
                    {move || error.get().map(|e| view! {
                        <div class="bg-red-900/50 border border-red-500 text-red-200 px-4 py-3 rounded">
                            {e}
                        </div>
                    })}

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            "Access Key"
                        </label>
                        <input
                            type="text"
                            class="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg
                                   text-white placeholder-gray-400 focus:outline-none focus:border-blue-500
                                   transition-colors"
                            placeholder="Enter access key"
                            prop:value=move || access_key.get()
                            on:input=move |ev| set_access_key.set(event_target_value(&ev))
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            "Secret Key"
                        </label>
                        <input
                            type="password"
                            class="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg
                                   text-white placeholder-gray-400 focus:outline-none focus:border-blue-500
                                   transition-colors"
                            placeholder="Enter secret key"
                            prop:value=move || secret_key.get()
                            on:input=move |ev| set_secret_key.set(event_target_value(&ev))
                        />
                    </div>

                    <button
                        type="submit"
                        class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 text-white font-medium
                               rounded-lg transition-colors focus:outline-none focus:ring-2
                               focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800
                               disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>

                <div class="mt-6 text-center text-sm text-gray-500">
                    "Default: minioadmin / minioadmin"
                </div>
            </div>
        </div>
    }
}
