use crate::model::Id;
use crate::model::user::User;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct PostMarker;

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct Post {
    pub id: Id<PostMarker>,
    pub author: User,
    pub content: String,
}
