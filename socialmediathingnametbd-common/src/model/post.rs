use crate::model::Id;
use crate::model::user::{User, UserMarker};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct PostMarker;

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct Post {
    pub id: Id<PostMarker>,
    pub author: User,
    pub content: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct CreatePost {
    pub author: Id<UserMarker>,
    pub content: String,
}
