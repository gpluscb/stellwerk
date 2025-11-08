create schema auth;

create table auth.auth_tokens
(
    user_snowflake        bigint    not null
        constraint auth_tokens_users_user_snowflake_fk
            references users.users,
    token                 char(30)  not null
        constraint auth_tokens_pk
            primary key,
    created_at            timestamp not null,
    expires_after_seconds bigint
);

comment on column auth.auth_tokens.created_at is 'UTC';

comment on column auth.auth_tokens.expires_after_seconds is 'If null, the token does not expire';
