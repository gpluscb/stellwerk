create schema users;

create table users.users
(
    snowflake bigint      not null
        constraint users_pk
            primary key,
    handle    varchar(50) not null
        constraint users_pk_2
            unique
);

create schema posts;

create table posts.posts
(
    snowflake bigint not null
        constraint posts_pk
            primary key,
    content   text   not null,
    author    bigint not null
        constraint posts_users_snowflake_fk
            references users.users
);
