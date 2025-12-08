//! Generic table component

use leptos::*;

/// Column definition for table (simplified for ease of use)
#[derive(Clone)]
pub struct TableColumn {
    pub header: &'static str,
    pub width: Option<&'static str>,
}

impl TableColumn {
    pub fn new(header: &'static str) -> Self {
        Self { header, width: None }
    }

    pub fn with_width(mut self, width: &'static str) -> Self {
        self.width = Some(width);
        self
    }
}

/// Simple table header component
#[component]
pub fn TableHeader(columns: Vec<&'static str>) -> impl IntoView {
    view! {
        <thead>
            <tr class="border-b border-gray-700">
                {columns.into_iter().map(|col| view! {
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        {col}
                    </th>
                }).collect_view()}
            </tr>
        </thead>
    }
}

/// Empty state component
#[component]
pub fn TableEmpty(message: &'static str, colspan: usize) -> impl IntoView {
    view! {
        <tr>
            <td colspan=colspan.to_string() class="px-4 py-8 text-center text-gray-400">
                {message}
            </td>
        </tr>
    }
}

/// Loading state component
#[component]
pub fn TableLoading(colspan: usize) -> impl IntoView {
    view! {
        <tr>
            <td colspan=colspan.to_string() class="px-4 py-8 text-center">
                <div class="flex items-center justify-center space-x-2">
                    <svg class="animate-spin h-5 w-5 text-blue-500" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <span class="text-gray-400">"Loading..."</span>
                </div>
            </td>
        </tr>
    }
}

/// Generic table wrapper
#[component]
pub fn Table(children: Children) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="w-full">
                {children()}
            </table>
        </div>
    }
}
