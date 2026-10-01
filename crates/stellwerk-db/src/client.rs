use crate::record::{
    AuthenticationRecord, Oauth2ProviderChoiceRecord, Oauth2StateRecord,
    Oauth2UserIdentityDiscordRecord, PartialPostRecord, PostRecord, UserRecord,
};
use sqlx::{PgPool, migrate, migrate::MigrateError, query, query_as, query_scalar};
use stellwerk_common::{
    model::{
        ModelValidationError,
        auth::{AuthTokenHash, Authentication},
        id::{Id, StellwerkSnowflakeGenerator},
        oauth2::{Oauth2State, Oauth2UserIdentityDiscord},
        pagination::PaginationReference,
        post::{PartialPost, Post, PostContent, PostMarker},
        user::{CreateUser, User, UserHandle, UserMarker},
    },
    snowflake::{ProcessId, WorkerId},
};
use thiserror::Error;
use time::{PrimitiveDateTime, UtcDateTime};

pub type Result<T, E = DbError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database migration failed: {0}")]
    Migrate(#[from] MigrateError),
    #[error("An object in the database was invalid: {0}")]
    Data(#[from] ModelValidationError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug)]
pub struct DbClient {
    pool: PgPool,
    snowflake_generator: StellwerkSnowflakeGenerator,
}

impl DbClient {
    pub async fn connect_and_migrate(
        url: &str,
        worker_id: WorkerId,
        process_id: ProcessId,
    ) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        migrate!().run(&pool).await?;

        Ok(Self::new(pool, worker_id, process_id))
    }

    #[must_use]
    pub fn new(pool: PgPool, worker_id: WorkerId, process_id: ProcessId) -> Self {
        let snowflake_generator = StellwerkSnowflakeGenerator::new(worker_id, process_id);

        Self {
            pool,
            snowflake_generator,
        }
    }

    pub async fn fetch_user(&self, user_id: Id<UserMarker>) -> Result<Option<User>> {
        let record = query_as!(
            UserRecord,
            "
            SELECT
                users.user_snowflake,
                users.handle
            FROM
                users.users
            WHERE
                users.user_snowflake = $1
            ",
            user_id.snowflake().get().cast_signed(),
        )
        .fetch_optional(&self.pool)
        .await?;

        let user = record.map(User::try_from).transpose()?;
        Ok(user)
    }

    pub async fn fetch_user_by_handle(&self, handle: &UserHandle) -> Result<Option<User>> {
        let record = query_as!(
            UserRecord,
            "
            SELECT
                users.user_snowflake,
                users.handle
            FROM
                users.users
            WHERE
                users.handle = $1
            ",
            handle.get(),
        )
        .fetch_optional(&self.pool)
        .await?;

        let user = record.map(User::try_from).transpose()?;
        Ok(user)
    }

    pub async fn fetch_user_posts(
        &self,
        user_id: Id<UserMarker>,
    ) -> Result<Option<Vec<PartialPost>>> {
        let mut transaction = self.pool.begin().await?;

        let user_exists = query_scalar!(
            r#"
            SELECT count(1) as "c!"
            FROM users.users
            WHERE users.user_snowflake = $1
            "#,
            user_id.snowflake().get().cast_signed(),
        )
        .fetch_one(&mut *transaction)
        .await?
            != 0;

        if !user_exists {
            return Ok(None);
        }

        let records = query_as!(
            PartialPostRecord,
            "
            SELECT
                posts.user_snowflake,
                posts.post_snowflake,
                posts.content
            FROM
                posts.posts
            WHERE
                posts.user_snowflake = $1
            ",
            user_id.snowflake().get().cast_signed(),
        )
        .fetch_all(&mut *transaction)
        .await?;

        let posts = records
            .into_iter()
            .map(PartialPost::try_from)
            .collect::<Result<_, _>>()?;

        Ok(Some(posts))
    }

    pub async fn create_user(&self, user: &CreateUser) -> Result<Id<UserMarker>> {
        let user_snowflake = self.snowflake_generator.generate();

        let returned_snowflake = query_scalar!(
            "
            INSERT INTO users.users (user_snowflake, handle)
            VALUES ($1, $2)
            RETURNING users.user_snowflake
            ",
            user_snowflake.get().cast_signed(),
            user.handle.get(),
        )
        .fetch_one(&self.pool)
        .await?;

        let returned_id: Id<UserMarker> = returned_snowflake.cast_unsigned().into();
        debug_assert_eq!(returned_id.snowflake(), user_snowflake);

        Ok(returned_id)
    }

    pub async fn fetch_post(&self, post_id: Id<PostMarker>) -> Result<Option<Post>> {
        let record = query_as!(
            PostRecord,
            "
            SELECT
                posts.post_snowflake,
                posts.content,
                users.user_snowflake,
                users.handle
            FROM
                posts.posts NATURAL JOIN users.users
            WHERE
                posts.post_snowflake = $1
            ",
            post_id.snowflake().get().cast_signed(),
        )
        .fetch_optional(&self.pool)
        .await?;

        let post = record.map(Post::try_from).transpose()?;
        Ok(post)
    }

    pub async fn fetch_recent_posts(
        &self,
        reference_post: PaginationReference,
        limit: u32,
    ) -> Result<Vec<Post>> {
        // TODO: Because we store the snowflake as an i64 in postgres, the comparisons will break in like uhhh twenty-ninety-something. Should fix before then.
        let records = match reference_post {
            PaginationReference::Newest => {
                query_as!(
                    PostRecord,
                    "
                    SELECT
                        posts.post_snowflake,
                        posts.content,
                        users.user_snowflake,
                        users.handle
                    FROM
                        posts.posts NATURAL JOIN users.users
                    ORDER BY
                        posts.post_snowflake
                    DESC
                    LIMIT $1
                    ",
                    i64::from(limit),
                )
                .fetch_all(&self.pool)
                .await?
            }
            PaginationReference::NewerThan { newer_than } => {
                let mut rows = query_as!(
                    PostRecord,
                    "
                    SELECT
                        posts.post_snowflake,
                        posts.content,
                        users.user_snowflake,
                        users.handle
                    FROM
                        posts.posts NATURAL JOIN users.users
                    WHERE
                        posts.post_snowflake > $1
                    ORDER BY
                        posts.post_snowflake
                    ASC
                    LIMIT $2
                    ",
                    newer_than.snowflake().get().cast_signed(),
                    i64::from(limit),
                )
                .fetch_all(&self.pool)
                .await?;
                // In the query we order by post snowflake ASC (oldest posts first),
                // so we need to reverse the ordering here
                rows.reverse();
                rows
            }
            PaginationReference::OlderThan { older_than } => {
                query_as!(
                    PostRecord,
                    "
                    SELECT
                        posts.post_snowflake,
                        posts.content,
                        users.user_snowflake,
                        users.handle
                    FROM
                        posts.posts NATURAL JOIN users.users
                    WHERE
                        posts.post_snowflake < $1
                    ORDER BY
                        posts.post_snowflake
                    DESC
                    LIMIT $2
                    ",
                    older_than.snowflake().get().cast_signed(),
                    i64::from(limit),
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        let posts = records
            .into_iter()
            .map(Post::try_from)
            .collect::<Result<_, _>>()?;
        Ok(posts)
    }

    pub async fn create_post(
        &self,
        content: &PostContent,
        author: Id<UserMarker>,
    ) -> Result<PartialPost> {
        let post_snowflake = self.snowflake_generator.generate();

        let returned_post = query_as!(
            PartialPostRecord,
            "
            INSERT INTO posts.posts (post_snowflake, content, user_snowflake)
            VALUES ($1, $2, $3)
            RETURNING post_snowflake, content, user_snowflake
            ",
            post_snowflake.get().cast_signed(),
            content.content,
            author.snowflake().get().cast_signed(),
        )
        .fetch_one(&self.pool)
        .await?
        .try_into()?;

        Ok(returned_post)
    }

    /// May return expired token
    pub async fn fetch_auth(&self, token_hash: &AuthTokenHash) -> Result<Option<Authentication>> {
        let record = query_as!(
            AuthenticationRecord,
            "
            SELECT
                auth_tokens.user_snowflake,
                auth_tokens.token_hash,
                auth_tokens.created_at,
                auth_tokens.expires_after_seconds
            FROM
                auth.auth_tokens
            WHERE
                auth_tokens.token_hash = $1
            ",
            &token_hash.0,
        )
        .fetch_optional(&self.pool)
        .await?;

        let authentication = record.map(Authentication::try_from).transpose()?;
        Ok(authentication)
    }

    pub async fn create_auth(&self, authentication: &Authentication) -> Result<()> {
        let created_at = PrimitiveDateTime::new(
            authentication.created_at.date(),
            authentication.created_at.time(),
        );

        query!(
            "
            INSERT INTO
                auth.auth_tokens (user_snowflake, token_hash, created_at, expires_after_seconds)
            VALUES
                ($1, $2, $3, $4)
            ",
            authentication.user.snowflake().get().cast_signed(),
            &authentication.token_hash.0,
            created_at,
            authentication
                .expires_after
                .map(|duration| duration.get().whole_seconds()),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Returns number of affected rows
    pub async fn drop_expired_tokens(&self) -> Result<u64> {
        let now_utc = UtcDateTime::now();
        let now_primitive = PrimitiveDateTime::new(now_utc.date(), now_utc.time());

        let rows_affected = query!(
            "
            DELETE FROM auth.auth_tokens
            WHERE auth_tokens.created_at
                      + make_interval(secs := auth_tokens.expires_after_seconds)
                      < $1
            ",
            now_primitive,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected)
    }

    /// May return expired oauth2 state
    pub async fn fetch_oauth2_state(&self, session_id: &str) -> Result<Option<Oauth2State>> {
        let record = sqlx::query_as!(
            Oauth2StateRecord,
            r#"
            SELECT
                session_id,
                auth_provider as "auth_provider: Oauth2ProviderChoiceRecord",
                csrf_token,
                redirect_url,
                expires_at
            FROM
                auth.oauth2_temp_states
            WHERE
                oauth2_temp_states.session_id = $1
            "#,
            session_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        let oauth2_state = record.map(Oauth2State::try_from).transpose()?;
        Ok(oauth2_state)
    }

    pub async fn create_oauth2_state(&self, oauth2_state: &Oauth2State) -> Result<Oauth2State> {
        let oauth2_provider_choice: Oauth2ProviderChoiceRecord = oauth2_state.auth_provider.into();
        let expires_at = PrimitiveDateTime::new(
            oauth2_state.expires_at.date(),
            oauth2_state.expires_at.time(),
        );

        let returned_oauth2_state = query_as!(
            Oauth2StateRecord,
            r#"
            INSERT INTO
                auth.oauth2_temp_states (session_id, auth_provider, csrf_token, redirect_url, expires_at)
            VALUES
                ($1, $2, $3, $4, $5)
            RETURNING
                session_id,
                auth_provider as "auth_provider: Oauth2ProviderChoiceRecord",
                csrf_token,
                redirect_url,
                expires_at
            "#,
            oauth2_state.session_id,
            oauth2_provider_choice as Oauth2ProviderChoiceRecord,
            oauth2_state.csrf_token.secret(),
            oauth2_state.redirect_url.as_str(),
            expires_at,
        )
        .fetch_one(&self.pool)
        .await?
        .try_into()?;

        Ok(returned_oauth2_state)
    }

    pub async fn delete_oauth2_state(&self, session_id: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            "
                    DELETE FROM auth.oauth2_temp_states
                    WHERE oauth2_temp_states.session_id = $1
                    ",
            session_id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Returns number of affected rows
    pub async fn drop_expired_oauth2_states(&self) -> Result<u64> {
        let now_utc = UtcDateTime::now();
        let now_primitive = PrimitiveDateTime::new(now_utc.date(), now_utc.time());

        let rows_affected = query!(
            "
            DELETE FROM auth.oauth2_temp_states
            WHERE oauth2_temp_states.expires_at < $1
            ",
            now_primitive,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected)
    }

    pub async fn fetch_oauth2_identity_discord(
        &self,
        discord_id: u64,
    ) -> Result<Option<Oauth2UserIdentityDiscord>> {
        let record = sqlx::query_as!(
            Oauth2UserIdentityDiscordRecord,
            r#"
            SELECT
                users.user_snowflake,
                users.oauth2_discord_id as "oauth2_discord_id!"
            FROM
                users.users
            WHERE
                oauth2_discord_id IS NOT NULL
              AND
                oauth2_discord_id = $1
            "#,
            discord_id.cast_signed(),
        )
        .fetch_optional(&self.pool)
        .await?;

        let identity = record
            .map(Oauth2UserIdentityDiscord::try_from)
            .transpose()?;
        Ok(identity)
    }
}
