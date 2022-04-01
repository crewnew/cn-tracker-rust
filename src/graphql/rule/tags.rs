#![cfg(feature = "graphql")]

#[derive(Serialize, Deserialize)]
pub struct IdValue {
    id: String,
    value: String
}

pub type Tag = IdValue;
pub type Regex = IdValue;
pub type Intrinsic = IdValue;
pub type Extrinsic = IdValue;

#[derive(Serialize, Deserialize)]
pub struct TagExtrinsic {
    id: String,
    tag: Tag,
    extrinsic: Extrinsic 
}

#[derive(Serialize, Deserialize)]
pub struct TagIntrinsic {
    id: String,
    tag: Tag,
    intrinsic: Intrinsic 
}

#[derive(Serialize, Deserialize)]
pub struct TagRegex {
    id: String,
    tag: Tag,
    regex: Regex 
}

