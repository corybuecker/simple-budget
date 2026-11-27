use crate::LayoutCtx;
use askama::Template;

/// Confirmation page shown before deleting a record. Shared by every section.
#[derive(Template)]
#[template(path = "delete_confirmation.html")]
pub struct DeleteConfirmation {
    pub layout: LayoutCtx,
    pub prompt: String,
    pub action: String,
    pub cancel: String,
    pub entity: String,
}
