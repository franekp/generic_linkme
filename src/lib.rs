mod distributed_fn_slice;

// Not public API.
#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub use generic_linkme_impl::*;

pub use crate::distributed_fn_slice::DistributedFnSlice;
