//! Button component

use leptos::*;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
}

impl ButtonVariant {
    fn class(&self) -> &'static str {
        match self {
            Self::Primary => "bg-blue-600 hover:bg-blue-700 text-white",
            Self::Secondary => "bg-gray-700 hover:bg-gray-600 text-white",
            Self::Danger => "bg-red-600 hover:bg-red-700 text-white",
            Self::Ghost => "bg-transparent hover:bg-gray-700 text-gray-300",
        }
    }
}

impl From<&str> for ButtonVariant {
    fn from(s: &str) -> Self {
        match s {
            "primary" => Self::Primary,
            "secondary" => Self::Secondary,
            "danger" => Self::Danger,
            "ghost" => Self::Ghost,
            _ => Self::Primary,
        }
    }
}

#[component]
pub fn Button(
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(into, optional)] loading: Option<Signal<bool>>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or_default();
    let extra_class = class.unwrap_or("");

    view! {
        <button
            class=format!(
                "px-4 py-2 rounded-lg font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center {} {}",
                variant.class(),
                extra_class
            )
            disabled=move || {
                disabled.map(|d| d.get()).unwrap_or(false) || loading.map(|l| l.get()).unwrap_or(false)
            }
            on:click=move |_| {
                if let Some(callback) = on_click {
                    callback.call(());
                }
            }
        >
            {move || loading.map(|l| l.get()).unwrap_or(false).then(|| view! {
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
            })}
            {children()}
        </button>
    }
}

/// Icon button (square, icon only)
#[component]
pub fn IconButton(
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(into)] on_click: Callback<()>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Ghost);

    view! {
        <button
            class=format!(
                "p-2 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed {}",
                variant.class()
            )
            disabled=move || disabled.map(|d| d.get()).unwrap_or(false)
            on:click=move |_| on_click.call(())
        >
            {children()}
        </button>
    }
}
