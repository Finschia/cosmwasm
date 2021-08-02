#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockStoreState {
    #[prost(int64, tag="1")]
    pub base: i64,
    #[prost(int64, tag="2")]
    pub height: i64,
}
