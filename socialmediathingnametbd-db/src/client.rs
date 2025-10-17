use crate::record::{FullPostRecord, PartialPostRecord, UserRecord};
use socialmediathingnametbd_common::{
    model::{
        Id, ModelValidationError, SocialmediathingnametbdSnowflakeGenerator,
        post::{CreatePost, PartialPost, Post, PostMarker},
        user::{CreateUser, User, UserHandle, UserMarker},
    },
    snowflake::{ProcessId, WorkerId},
};
use sqlx::{PgPool, migrate, migrate::MigrateError, query_as, query_scalar};
use std::sync::nonpoison::Mutex;
use thiserror::Error;

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
    snowflake_generator: Mutex<SocialmediathingnametbdSnowflakeGenerator>,
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
        let snowflake_generator = Mutex::new(SocialmediathingnametbdSnowflakeGenerator::new(
            worker_id, process_id,
        ));

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
        let user_snowflake = self.snowflake_generator.lock().generate();

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
            FullPostRecord,
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

    pub async fn create_post(&self, post: &CreatePost) -> Result<Id<PostMarker>> {
        let post_snowflake = self.snowflake_generator.lock().generate();

        let returned_snowflake = query_scalar!(
            "
            INSERT INTO posts.posts (post_snowflake, content, user_snowflake)
            VALUES ($1, $2, $3)
            RETURNING posts.post_snowflake
            ",
            post_snowflake.get().cast_signed(),
            post.content,
            post.author.snowflake().get().cast_signed(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(returned_snowflake.cast_unsigned().into())
    }
}
