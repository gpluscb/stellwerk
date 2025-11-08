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
Ensuring independence from commercial intersts is also important to us and we would love to get federation working.

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

### Running in Docker

1. Install Docker.
2. Create a directory `docker/Secrets` with files `POSTGRES_USER.txt`, `POSTGRES_PASSWORD.txt`, `POSTGRES_DB.txt`.
   The user and password files should contain the name and password your postgres user should have.
   The contents of the db file will be the name of the database, for example `stellwerk`.
3. Create the env file `docker/stellwerk-api/.env`.
4. Run `docker compose --file docker/docker-compose.yml up --build`.
   You can re-run this command whenever you make new changes.

### Running with Cargo

1. Install rust (nightly).
2. Create the env file `stellwerk-api/.env`
3. cd into `stellwerk-api`
4. Run `cargo run`

### Example `.env`:

```.env
SERVER_ADDRESS=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=postgres://postgresuser:postgrespw@127.0.0.1/postgresdb
WORKER_ID=0
PROCESS_ID=0
```
