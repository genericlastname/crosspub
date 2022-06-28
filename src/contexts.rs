use serde::Serialize;

use crate::about::About;
use crate::post::Post;
use crate::topic::Topic;
use crate::config::Site;

#[derive(Serialize)]
pub struct PostContext {
    pub site: Site,
    pub post: Post,
    pub has_about: bool,
}

#[derive(Serialize)]
pub struct TopicContext {
    pub site: Site,
    pub topic: Topic,
    pub has_about: bool,
}

#[derive(Serialize)]
pub struct IndexContext {
    pub site: Site,
    pub posts: Vec<Post>,
    pub latest_post: Post,
    pub topics: Vec<Topic>,
    pub has_topics: bool,
    pub has_about: bool,
}

#[derive(Serialize)]
pub struct AboutContext {
    pub site: Site,
    pub about: About,
    pub has_about: bool,
}

#[derive(Serialize)]
pub struct AtomFeedContext {
    pub site: Site,
    pub latest_post: Post,
    pub entries: Vec<String>,
}

#[derive(Serialize)]
pub struct AtomEntryContext {
    pub site: Site,
    pub post: Post,
}
