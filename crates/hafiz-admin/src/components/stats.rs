//! Statistics card component

use leptos::*;

#[component]
pub fn StatCard(
    title: &'static str,
    value: String,
    #[prop(optional)] subtitle: Option<&'static str>,
    #[prop(optional)] color: Option<&'static str>,
) -> impl IntoView {
    let color = color.unwrap_or("blue");
    
    let bg_class = match color {
        "blue" => "bg-blue-600/20",
        "green" => "bg-green-600/20",
        "purple" => "bg-purple-600/20",
        "orange" => "bg-orange-600/20",
        "red" => "bg-red-600/20",
        _ => "bg-gray-600/20",
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
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                        </svg>
                    </div>
                </div>
            </div>
        </div>
    }
}
