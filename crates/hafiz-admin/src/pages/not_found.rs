//! 404 Not Found page

use leptos::*;
use crate::components::Button;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    let go_home = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/");
        }
    };

    view! {
        <div class="min-h-[60vh] flex items-center justify-center">
            <div class="text-center">
                <div class="mb-8">
                    <svg class="w-32 h-32 mx-auto text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1"
                            d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                </div>
                <h1 class="text-6xl font-bold text-white mb-4">"404"</h1>
                <h2 class="text-2xl font-semibold text-gray-300 mb-2">"Page Not Found"</h2>
                <p class="text-gray-400 mb-8 max-w-md mx-auto">
                    "The page you're looking for doesn't exist or has been moved."
                </p>
                <div class="flex items-center justify-center space-x-4">
                    <Button on_click=Callback::new(go_home)>
                        "Go to Dashboard"
                    </Button>
                    <a href="/buckets" class="px-4 py-2 text-gray-300 hover:text-white transition-colors">
                        "View Buckets"
                    </a>
                </div>
            </div>
        </div>
    }
}
