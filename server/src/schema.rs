// @generated automatically by Diesel CLI.

diesel::table! {
    entries (id) {
        id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    updates (id) {
        id -> Uuid,
        entry_id -> Uuid,
        created -> Timestamp,
        aes_key -> Bytea,
        sig -> Bytea,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Text,
        password -> Bytea,
        salt -> Text,
        public_key -> Bytea,
    }
}

diesel::joinable!(entries -> users (user_id));
diesel::joinable!(updates -> entries (entry_id));

diesel::allow_tables_to_appear_in_same_query!(
    entries,
    updates,
    users,
);
