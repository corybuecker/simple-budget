mod home;
mod navigation;
use navigation::Navigation;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html! {
        <Navigation />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
