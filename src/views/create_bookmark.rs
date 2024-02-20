use askama::Template;

use crate::{form_errors::FormErrors, forms};

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "create_bookmark.html")]
pub struct CreateBookmarkTemplate {
    pub layout: LayoutTemplate,

    pub errors: FormErrors,
    pub input: forms::bookmarks::CreateBookmark,
}