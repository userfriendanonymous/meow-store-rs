
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

binbuf::fixed! {
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, bincode::Encode, bincode::Decode)]
    pub struct Value {
        len: u8,
        content: [u8; 20],
    }

    buf! { pub struct ValueBuf<P>(Value, P); }

    impl I for Value {
        type Buf<P> = ValueBuf<P>;
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        Ok(
            Self::from_str(&String::deserialize(deserializer)?)
                .map_err(|_| serde::de::Error::custom("Fail"))?
        )
    }
}

impl Value {
    pub unsafe fn from_raw(len: u8, content: [u8; 20]) -> Self {
        Self { len, content }
    }
    // Sorted array
    pub const CHARS: [char; 64] = [
        '-',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'A', 'B', 'C', 'D', 'E', 
        'F', 'G', 'H', 'I', 'J', 
        'K', 'L', 'M', 'N', 'O',
        'P', 'Q', 'R', 'S', 'T', 
        'U', 'V', 'W', 'X', 'Y', 'Z',
        '_',
        'a', 'b', 'c', 'd', 'e', 
        'f', 'g', 'h', 'i', 'j', 
        'k', 'l', 'm', 'n', 'o',
        'p', 'q', 'r', 's', 't', 
        'u', 'v', 'w', 'x', 'y', 'z',
    ];


    // fn char(self, idx: u8) -> u8 {
    //     let mut n = u128::from_be_bytes(self.0);
    //     n >>= 6 * idx;
    //     n.to_be_bytes()[14]
    // }
}

impl FromStr for Value {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 20 {
            Err(())?
        }
        let mut content = [0; 20];
        for (idx, ch) in s.chars().enumerate() {
            if let Ok(id) = Self::CHARS.binary_search(&ch) {
                content[idx] = id as u8;
                // content |= (id as u128) << (8 + 6 * id);
            } else {
                Err(())?
            }
        }
        Ok(Self { len: s.len() as u8, content })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::with_capacity(20);
        for idx in 0 .. self.len as usize {
            let ch = Self::CHARS[self.content[idx] as usize];
            string.push(ch);
        }
        f.write_str(&string)
    }
}

impl binbuf::Fixed for Value {
    const LEN: usize = 16;
    fn encode(&self, buf: binbuf::BufMut<Self>) {
        let bytes = buf.0.slice();
        bytes.copy_from_slice(const { &[0; 16] });
        unsafe {
            let mut byte_idx = 15;
            *bytes.get_unchecked_mut(byte_idx) = self.len;
            // byte_idx -= 1;
            let mut state = 0u8;
            for idx in 0 .. self.len as usize {
                let ch = *self.content.get_unchecked(idx);
                if state == 0 {
                    byte_idx -= 1;
                    *bytes.get_unchecked_mut(byte_idx) |= ch;
                } else if state == 1 {
                    *bytes.get_unchecked_mut(byte_idx) |= ch << 6;
                    byte_idx -= 1;
                    *bytes.get_unchecked_mut(byte_idx) |= ch >> 2;
                } else if state == 2 {
                    *bytes.get_unchecked_mut(byte_idx) |= ch << 4;
                    byte_idx -= 1;
                    *bytes.get_unchecked_mut(byte_idx) |= ch >> 4;
                } else {
                    // byte_idx -= 1;
                    *bytes.get_unchecked_mut(byte_idx) |= ch << 2;
                }
                state = (state + 1) % 4;
            }
        }
    }
}

impl binbuf::fixed::Decode for Value {
    fn decode(buf: binbuf::BufConst<Self>) -> Self {
        let mut content = [0; 20];
        let bytes = buf.0.slice();
        unsafe {
            let mut byte_idx = 15;
            let len = *bytes.get_unchecked(byte_idx);
            // byte_idx -= 1;
            let mut state = 0u8;
            let mut ch = 0u8;
            for idx in 0 .. len as usize {
                if state == 0 {
                    byte_idx -= 1;
                    ch = (*bytes.get_unchecked(byte_idx) << 2) >> 2;
                } else if state == 1 {
                    ch = *bytes.get_unchecked(byte_idx) >> 6;
                    byte_idx -= 1;
                    ch |= (*bytes.get_unchecked(byte_idx) << 4) >> 2;
                } else if state == 2 {
                    ch = *bytes.get_unchecked(byte_idx) >> 4;
                    byte_idx -= 1;
                    ch |= (*bytes.get_unchecked(byte_idx) << 6) >> 2;
                } else {
                    // byte_idx -= 1;
                    ch = *bytes.get_unchecked(byte_idx) >> 2;
                }
                *content.get_unchecked_mut(idx) = ch;
                state = (state + 1) % 4;
            }
            Self { len, content }
        }
    }
}