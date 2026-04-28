create table tasks (
    task_id serial primary key,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    task_start_date timestamp not null,
    task_end_date timestamp not null,
    task_description text not null,
    task_title text not null,
    task_priority varchar(10) not null
);