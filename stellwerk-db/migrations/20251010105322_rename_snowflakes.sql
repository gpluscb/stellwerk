alter table posts.posts
    rename column snowflake to post_snowflake;

alter table posts.posts
    rename column author to user_snowflake;

alter table users.users
    rename column snowflake to user_snowflake;
