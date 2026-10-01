# Stellwerk

Stellwerk is meant to become a proof of concept social media site where instead of algorithms, users decide what appears
on their feeds.

## Philosophy

In the wake of Twitter's fall from grace, some new social media platforms have gained traction.
But frustratingly, these platforms adopt many "twitterisms", mechanics that in our opinion obstruct community building.
Instead, we want to borrow some community-building and community-finding ideas from elsewhere, for example from Tumblr
with a strong tagging and reblog-style comment system.

We oppose algorithmic feeds and instead want to give more tools to users to customise their feeds while keeping them
deterministic and predictable.
Ensuring independence from commercial interests is also important to us, and we would love to get federation working.

## Current Status

Some scaffolding for the backend is there, but no real functionality is implemented yet.

## Looking for Contributors

The project is organised on the [codeblr Discord server](https://discord.gg/BrZ5GV8gzs).
Feel free to join if you want to contribute or are just curious about the project.

## Architecture

This project is in an extremely early stage and none of the architecture is completely set in stone.

For the time being, the most basic planned architecture is:

```mermaid
architecture-beta
    group backend[Backend]
    group frontend[Frontend]

    service db(database)[Database] in backend
    service api(server)[REST API] in backend

    service web(server)[Web server] in frontend

    db:L -- R:api
    web:L -- R:api
```

The REST API could be public.

The frontend does not exist yet and technologies for the frontend are not decided yet.
The REST API server `stellwerk-api` is written in Rust (nigthly for fun) with Axum.
The database connection between api and the db is achieved with `stellwerk-db`.
The database is PostgreSQL and the whole thing can be coordinated using Docker.

### IDs

The project uses Twitter snowflakes.
See the [Discord developer docs](https://discord.com/developers/docs/reference#snowflakes) for an explanation.
The `Id<Marker>` type is a type checked `StellwerkSnowflake`.
Its only purpose is to ensure that, for example, a user id is not accidentally used where a post id is asked for.

## Setup and Building

### Running with Docker (recommended)

1. Install Docker.
2. Create a directory `docker/postgres/secrets` with files `POSTGRES_USER.dev.txt`, `POSTGRES_PASSWORD.dev.txt`, and
   `POSTGRES_DB.dev.txt`.
   The user and password files should contain the name and password your postgres user should have.
   The contents of the db file will be the name of the database, for example `stellwerk`.
3. Create the api config file `docker/stellwerk-api/secrets/config.dev.toml`.
4. Run `docker compose --file docker/docker-compose.dev.yml up --build`.
   Changes to the `stellwerk-frontend` are applied automatically.
   To apply changes to `stellwerk-api`, you can run
   `docker compose --file docker/docker-compose.dev.yml up --build stellwerk-api`.

### Running with not Docker (not recommended)

1. Install PostgreSQL and create a PostgreSQL database
2. Install rust (nightly).
3. Create a `config.toml` file for stellwerk-api
4. cd into `stellwerk-api`
5. Run `cargo run` with the `STELLWERK_API_CONFIG_FILE` argument pointing to your `config.toml` file
6. cd into `stellwerk-frontend`
7. Run `npm run dev`

### Example `config.toml` for stellwerk-api:

```toml
server_address = "127.0.0.1:8080"
database_url = "postgres://postgresuser:postgrespw@127.0.0.1/postgresdb"
worker_id = 0
process_id = 0

[oauth2_providers_config.discord]
client_id = "332269999912132097"
client_secret = "937it3ow87i4ery69876wqire"
auth_url = "https://discord.com/oauth2/authorize"
token_url = "https://discord.com/api/v10/oauth2/token"
revocation_url = "https://discord.com/api/v10/oauth2/token/revoke"
scopes = ["identify"]

[login_logout_config]
expiring_token_duration_seconds = 86400 # One day
```
