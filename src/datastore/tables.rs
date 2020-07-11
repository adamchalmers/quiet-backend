#[allow(unused_imports)]
use diesel::sql_types::*;

table! {
    use crate::datastore::structs::ContentMapping;
    #[allow(unused_imports)]
    use diesel::sql_types::*;
    posts (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        content -> ContentMapping,
        text -> Text,
        user_id -> Uuid,
    }
}

table! {
    users (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        name -> Text,
    }
}

table! {
    follows (posts, reads) {
        posts -> Uuid,
        reads -> Uuid,
    }
}

joinable!(posts -> users (user_id));
allow_tables_to_appear_in_same_query!(posts, users);

allow_tables_to_appear_in_same_query!(follows, users);
