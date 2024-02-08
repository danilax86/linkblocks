use crate::{
    authentication::AuthUser,
    db::{self, ExtractTx},
    response_error::ResponseResult,
    views::{index::IndexTemplate, layout::LayoutTemplate},
};
use axum::{routing::get, Router};
use sqlx::{Pool, Postgres};

pub fn router() -> Router<Pool<Postgres>> {
    Router::new().route("/", get(index))
}

async fn index(auth_user: AuthUser, ExtractTx(mut tx): ExtractTx) -> ResponseResult<IndexTemplate> {
    let user = db::users::by_id(&mut tx, auth_user.user_id).await?;
    let lists = db::notes::list_pinned_by_user(&mut tx, user.id).await?;

    Ok(IndexTemplate {
        layout: LayoutTemplate {
            logged_in_username: user.username,
            notes: lists,
        },
    })
}
