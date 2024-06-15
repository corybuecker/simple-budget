use crate::home;
use home::*;
use yew::{function_component, html, Html};
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/accounts")]
    Accounts,
    #[at("/savings")]
    Savings,
    #[at("/goals")]
    Goals,
    #[not_found]
    #[at("/404")]
    NotFound,
}
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home/> },
        Route::Accounts => html! { <Accounts/> },
        Route::Savings => html! { <Savings/> },
        Route::Goals => html! { <Goals/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component]
pub fn Navigation() -> Html {
    html! {
       <BrowserRouter>
           <Switch<Route> render={switch} />
           <nav>
                <Link<Route> to={Route::Home}>{ "Dashboard" }</Link<Route>>
                <Link<Route> to={Route::Accounts}>{ "Accounts" }</Link<Route>>
                <Link<Route> to={Route::Savings}>{ "Savings" }</Link<Route>>
                <Link<Route> to={Route::Goals}>{ "Goals" }</Link<Route>>
            </nav>
       </BrowserRouter>
    }
}
