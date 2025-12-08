//! Header component with search and user menu

use leptos::*;

#[component]
pub fn Header() -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let (show_user_menu, set_show_user_menu) = create_signal(false);

    let on_logout = move |_| {
        // Clear credentials
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.remove_item("hafiz_access_key");
            let _ = storage.remove_item("hafiz_secret_key");
        }
        
        // Redirect to login
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/login");
        }
    };

    view! {
        <header class="h-16 bg-gray-900 border-b border-gray-700 flex items-center justify-between px-6">
            // Search
            <div class="flex-1 max-w-xl">
                <div class="relative">
                    <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" 
                         fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                    <input
                        type="text"
                        class="w-full pl-10 pr-4 py-2 bg-gray-800 border border-gray-700 rounded-lg 
                               text-white placeholder-gray-400 focus:outline-none focus:border-blue-500
                               transition-colors"
                        placeholder="Search buckets, objects..."
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                </div>
            </div>

            // Right side actions
            <div class="flex items-center space-x-4">
                // Notifications
                <button class="p-2 text-gray-400 hover:text-white transition-colors rounded-lg hover:bg-gray-800">
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                    </svg>
                </button>

                // User menu
                <div class="relative">
                    <button 
                        class="flex items-center space-x-3 p-2 rounded-lg hover:bg-gray-800 transition-colors"
                        on:click=move |_| set_show_user_menu.update(|v| *v = !*v)
                    >
                        <div class="w-8 h-8 bg-blue-600 rounded-full flex items-center justify-center">
                            <span class="text-sm font-medium text-white">"A"</span>
                        </div>
                        <span class="text-sm text-gray-300">"Admin"</span>
                        <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                    </button>

                    // Dropdown menu
                    {move || show_user_menu.get().then(|| view! {
                        <div class="absolute right-0 mt-2 w-48 bg-gray-800 rounded-lg shadow-lg border border-gray-700 py-1 z-50">
                            <a href="/settings" class="block px-4 py-2 text-sm text-gray-300 hover:bg-gray-700">
                                "Settings"
                            </a>
                            <hr class="my-1 border-gray-700" />
                            <button 
                                class="block w-full text-left px-4 py-2 text-sm text-red-400 hover:bg-gray-700"
                                on:click=on_logout
                            >
                                "Sign Out"
                            </button>
                        </div>
                    })}
                </div>
            </div>
        </header>
    }
}
