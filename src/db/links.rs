use sqlx::{query, query_as, FromRow, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    app_error::AppResult,
    db,
    schemas::links::{CreateLink, ReferenceType},
};

#[derive(FromRow)]
pub struct Link {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub src_bookmark_id: Option<Uuid>,
    pub src_note_id: Option<Uuid>,
    pub src_list_id: Option<Uuid>,

    pub dest_bookmark_id: Option<Uuid>,
    pub dest_note_id: Option<Uuid>,
    pub dest_list_id: Option<Uuid>,
}

pub enum LinkDestination {
    Bookmark(db::Bookmark),
    Note(db::Note),
    List(db::List),
}

pub struct LinkWithContent {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub dest: LinkDestination,
}

pub async fn insert(
    db: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    create: CreateLink,
) -> AppResult<Link> {
    let src_bookmark_id = (create.src_ref_type == ReferenceType::Bookmark).then_some(create.src_id);
    let src_note_id = (create.src_ref_type == ReferenceType::Note).then_some(create.src_id);
    let src_list_id = (create.src_ref_type == ReferenceType::List).then_some(create.src_id);

    let dest_bookmark_id =
        (create.dest_ref_type == ReferenceType::Bookmark).then_some(create.dest_id);
    let dest_note_id = (create.dest_ref_type == ReferenceType::Note).then_some(create.dest_id);
    let dest_list_id = (create.dest_ref_type == ReferenceType::List).then_some(create.dest_id);

    let list = query_as!(
        Link,
        r#"
        insert into links
        (
            user_id,
            src_bookmark_id,
            src_note_id,
            src_list_id,
            dest_bookmark_id,
            dest_note_id,
            dest_list_id
        )
        values ($1, $2, $3, $4, $5, $6, $7)
        returning *"#,
        user_id,
        src_bookmark_id,
        src_note_id,
        src_list_id,
        dest_bookmark_id,
        dest_note_id,
        dest_list_id
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(list)
}

pub async fn list_by_list(
    db: &mut Transaction<'_, Postgres>,
    list_id: Uuid,
) -> AppResult<Vec<LinkWithContent>> {
    let rows = query!(
        r#"
        select 
            links.id as link_id, 
            links.created_at as link_created_at, 
            links.user_id as link_user_id,

            coalesce(notes.id, bookmarks.id, lists.id) as "dest_id!",
            coalesce(notes.created_at, bookmarks.created_at, lists.created_at) as "dest_created_at!", 
            coalesce(notes.user_id, bookmarks.user_id, lists.user_id) as "dest_user_id!",

            notes.content as "note_content?",
            bookmarks.url as "bookmark_url?",
            bookmarks.title as "bookmark_title?",
            lists.title as "list_title?"
        from links
        left join notes on notes.id = links.dest_note_id
        left join bookmarks on bookmarks.id = links.dest_bookmark_id
        left join lists on lists.id = links.dest_list_id
        where links.src_list_id = $1
        -- temporary hack for random ordering of demo data
        order by link_id
        "#,
        list_id
    )
    .fetch_all(&mut **db)
    .await?;

    let results = rows
        .into_iter()
        .map(|row| {
            let dest = if let Some(content) = row.note_content {
                LinkDestination::Note(db::Note {
                    id: row.dest_id,
                    created_at: row.dest_created_at,
                    user_id: row.dest_user_id,
                    content,
                })
            } else if let (Some(url), Some(title)) = (row.bookmark_url, row.bookmark_title) {
                LinkDestination::Bookmark(db::Bookmark {
                    id: row.dest_id,
                    created_at: row.dest_created_at,
                    user_id: row.dest_user_id,
                    url,
                    title,
                })
            } else if let Some(title) = row.list_title {
                LinkDestination::List(db::List {
                    id: row.dest_id,
                    created_at: row.dest_created_at,
                    user_id: row.dest_user_id,
                    title,
                })
            } else {
                return Err(anyhow::anyhow!(
                    "Don't know how to convert row into LinkDestination struct"
                ));
            };
            Ok(LinkWithContent {
                id: row.link_id,
                created_at: row.link_created_at,
                user_id: row.link_user_id,
                dest,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(results)
}
