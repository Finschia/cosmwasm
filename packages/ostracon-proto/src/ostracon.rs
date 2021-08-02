//! Ostracon-proto auto-generated sub-modules for Ostracon

pub mod privval {
    include!("prost/ostracon.privval.rs");
}

pub mod rpc {
    pub mod grpc {
        include!("prost/ostracon.rpc.grpc.rs");
    }
}

pub mod state {
    include!("prost/ostracon.state.rs");
}

pub mod libs {
    pub mod bits {
        include!("prost/ostracon.libs.bits.rs");
    }
}

pub mod abci {
    include!("prost/ostracon.abci.rs");
}

pub mod mempool {
    include!("prost/ostracon.mempool.rs");
}

pub mod p2p {
    include!("prost/ostracon.p2p.rs");
}

pub mod blockchain {
    include!("prost/ostracon.blockchain.rs");
}

pub mod statesync {
    include!("prost/ostracon.statesync.rs");
}

pub mod version {
    include!("prost/ostracon.version.rs");
}

pub mod types {
    include!("prost/ostracon.types.rs");
}

pub mod store {
    include!("prost/ostracon.store.rs");
}

pub mod consensus {
    include!("prost/ostracon.consensus.rs");
}

pub mod crypto {
    include!("prost/ostracon.crypto.rs");
}

pub mod meta {
    pub const REPOSITORY: &str = "https://github.com/line/ostracon";
    pub const COMMITISH: &str = "a727fe1db4e7e7f7037e15b61b76bab2d01de829";
}
