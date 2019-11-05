extern crate byteorder;
extern crate crypto;
extern crate hex;

pub use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
pub use stream::{Deserializable, Serializable, Stream};
pub mod error;
pub mod header;
pub mod stream;

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate salticidae_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use salticidae_derive::Deserialize;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use salticidae_derive::Serialize;

#[cfg(test)]
mod tests {}
