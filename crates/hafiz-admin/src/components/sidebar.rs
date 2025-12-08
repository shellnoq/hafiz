//! Sidebar navigation component

use leptos::*;
use leptos_router::use_location;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    
    let is_active = move |path: &str| {
        let current = location.pathname.get();
        if path == "/" {
            current == "/"
        } else {
            current.starts_with(path)
        }
    };

    view! {
        <aside class="w-64 bg-gray-900 border-r border-gray-700 flex flex-col">
            // Logo
            <div class="h-16 flex items-center px-6 border-b border-gray-700">
                <a href="/" class="flex items-center space-x-3">
                    <svg class="w-8 h-8 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                    </svg>
                    <span class="text-xl font-bold text-white">"Hafiz"</span>
                </a>
            </div>

            // Navigation
            <nav class="flex-1 px-4 py-6 space-y-2">
                <NavItem 
                    href="/" 
                    icon=IconDashboard 
                    label="Dashboard" 
                    active=Signal::derive(move || is_active("/") && !is_active("/buckets") && !is_active("/users") && !is_active("/cluster") && !is_active("/settings"))
                />
                <NavItem 
                    href="/buckets" 
                    icon=IconBuckets 
                    label="Buckets" 
                    active=Signal::derive(move || is_active("/buckets"))
                />
                <NavItem 
                    href="/users" 
                    icon=IconUsers 
                    label="Users" 
                    active=Signal::derive(move || is_active("/users"))
                />
                <NavItem 
                    href="/cluster" 
                    icon=IconCluster 
                    label="Cluster" 
                    active=Signal::derive(move || is_active("/cluster"))
                />
                
                <div class="pt-4 mt-4 border-t border-gray-700">
                    <NavItem 
                        href="/settings" 
                        icon=IconSettings 
                        label="Settings" 
                        active=Signal::derive(move || is_active("/settings"))
                    />
                </div>
            </nav>

            // Version info
            <div class="px-6 py-4 border-t border-gray-700">
                <div class="text-xs text-gray-500">
                    <div>"Hafiz v0.1.0"</div>
                    <div class="mt-1">"S3 API Compatible"</div>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn NavItem(
    href: &'static str,
    icon: fn() -> impl IntoView,
    label: &'static str,
    active: Signal<bool>,
) -> impl IntoView {
    view! {
        <a
            href=href
            class=move || {
                let base = "flex items-center px-4 py-3 rounded-lg transition-colors";
                if active.get() {
                    format!("{} bg-blue-600 text-white", base)
                } else {
                    format!("{} text-gray-400 hover:bg-gray-800 hover:text-white", base)
                }
            }
        >
            {icon()}
            <span class="ml-3">{label}</span>
        </a>
    }
}

#[component]
fn IconDashboard() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
        </svg>
    }
}

#[component]
fn IconBuckets() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
        </svg>
    }
}

#[component]
fn IconUsers() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
    }
}

#[component]
fn IconSettings() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
    }
}

#[component]
fn IconCluster() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
    }
}
