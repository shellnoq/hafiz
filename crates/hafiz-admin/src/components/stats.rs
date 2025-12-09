//! Statistics card component

use leptos::*;

#[component]
pub fn StatCard(
    title: &'static str,
    value: String,
    #[prop(optional)] subtitle: Option<&'static str>,
    #[prop(optional)] color: Option<&'static str>,
    #[prop(optional)] icon: Option<&'static str>,
) -> impl IntoView {
    let color = color.unwrap_or("blue");

    let bg_class = match color {
        "blue" => "bg-blue-600/20",
        "green" => "bg-green-600/20",
        "purple" => "bg-purple-600/20",
        "orange" => "bg-orange-600/20",
        "red" => "bg-red-600/20",
        "yellow" => "bg-yellow-600/20",
        _ => "bg-gray-600/20",
    };

    let icon_path = match icon.unwrap_or("chart") {
        "server" => "M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2",
        "check-circle" => "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
        "x-circle" => "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
        "clock" => "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
        "database" => "M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4",
        "users" => "M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z",
        "folder" => "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z",
        _ => "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
    };

    view! {
        <div class="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <div class="flex items-start justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-400">{title}</p>
                    <p class="text-3xl font-bold text-white mt-2">{value}</p>
                    {subtitle.map(|s| view! {
                        <p class="text-sm text-gray-500 mt-1">{s}</p>
                    })}
                </div>
                <div class=format!("p-3 rounded-lg {}", bg_class)>
                    <div class="w-5 h-5 text-white opacity-80">
                        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=icon_path />
                        </svg>
                    </div>
                </div>
            </div>
        </div>
    }
}
