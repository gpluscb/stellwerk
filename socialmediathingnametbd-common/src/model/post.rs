use crate::model::{
    Id,
    user::{User, UserMarker},
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct PostMarker;

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, Deserialize, Serialize)]
pub struct Post {
    pub id: Id<PostMarker>,
    pub author: User,
    pub content: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, Deserialize, Serialize)]
pub struct CreatePost {
    pub author: Id<UserMarker>,
    pub content: String,
}
