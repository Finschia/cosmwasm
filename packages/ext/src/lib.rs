mod collection;
mod msg;
mod msg_collection;
mod msg_token;
mod querier_collection;
mod querier_token;
mod query;
mod token;

pub use collection::{
    Coin, Collection, CollectionPerm, MintNFTParam, Token as CollectionToken, TokenType,
};
pub use msg::{Change, LinkMsgWrapper, Module, MsgData};
pub use msg_collection::{CollectionMsg, CollectionRoute};
pub use msg_token::{TokenMsg, TokenRoute};
pub use querier_collection::{CollectionQuery, CollectionQueryRoute, LinkCollectionQuerier};
pub use querier_token::{LinkTokenQuerier, TokenQuery, TokenQueryRoute};
pub use query::{LinkQueryWrapper, QueryData, Response, Target};
pub use token::{Token, TokenPerm};

// This export is added to all contracts that import this package, signifying that they require
// "link" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_link() {}
