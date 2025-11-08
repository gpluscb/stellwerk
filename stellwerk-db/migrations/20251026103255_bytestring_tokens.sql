alter table auth.auth_tokens
    alter column token type bytea using token::bytea;

alter table auth.auth_tokens
    rename column token to token_hash;
