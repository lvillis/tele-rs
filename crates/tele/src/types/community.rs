use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Community {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct CommunityChatAdded {
    pub community: Community,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct CommunityChatJoined {
    pub community: Community,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct CommunityChatRemoved {}
