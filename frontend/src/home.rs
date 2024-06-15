use yew::{function_component, html, Html};

#[function_component]
pub fn Home() -> Html {
    html! { <p>{"Hello world"}</p> }
}

#[function_component]
pub fn Accounts() -> Html {
    html! { <p>{"Hello accounts"}</p> }
}

#[function_component]
pub fn Savings() -> Html {
    html! { <p>{"Hello savings"}</p> }
}

#[function_component]
pub fn Goals() -> Html {
    html! { <p>{"Hello goals"}</p> }
}
