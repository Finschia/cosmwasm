mod msg;
mod msg_token;
mod msg_collection;

pub use msg::{Module, MsgData, LinkMsgWrapper, Change};
pub use msg_token::{TokenRoute, TokenMsg};
pub use msg_collection::{CollectionRoute, CollectionMsg};

// This export is added to all contracts that import this package, signifying that they require
// "link" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_link() {}
