mod msg;
mod msg_collection;
mod msg_token;

pub use msg::{Change, LinkMsgWrapper, Module, MsgData};
pub use msg_collection::{CollectionMsg, CollectionRoute};
pub use msg_token::{TokenMsg, TokenRoute};

// This export is added to all contracts that import this package, signifying that they require
// "link" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_link() {}
