
use std::{fmt::Display, str::FromStr};

use bytedb::entry::Ptr;

binbuf::fixed! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Value(u16);

    buf! { pub struct ValueBuf<P>(Value, P); }

    impl I for Value {
        type Buf<P> = ValueBuf<P>;
    }
    impl Code for Value {}
}