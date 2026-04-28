// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (task_id) {
        task_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        task_start_date -> Timestamp,
        task_end_date -> Timestamp,
        task_description -> Text,
        task_title -> Text,
        #[max_length = 10]
        task_priority -> Varchar,
    }
}
