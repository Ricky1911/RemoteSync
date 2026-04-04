// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Text,
        password -> Bytea,
        salt -> Text,
        public_key -> Bytea,
    }
}
