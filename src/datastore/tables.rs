table! {
    use crate::datastore::ContentMapping;
    #[allow(unused_imports)]
    use diesel::sql_types::*;
    posts (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        content -> ContentMapping,
        text -> Text,
        account_id -> Uuid,
    }
}
