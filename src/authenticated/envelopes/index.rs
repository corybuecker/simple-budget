use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse,
    models::envelope::Envelope, utilities::responses::generate_response,
};
use askama::Template;
use axum::{Extension, extract::State, http::StatusCode};

#[derive(Template)]
#[template(path = "envelopes/index.html")]
struct Index {
    #[allow(unused)]
    layout: LayoutCtx,
    envelopes: Vec<Envelope>,
}

pub async fn action(
    shared_state: State<SharedState>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let envelopes = Envelope::get_all(&client, user.id).await?;
    let template = Index { layout, envelopes };

    Ok(generate_response(template.render()?, StatusCode::OK))
}
