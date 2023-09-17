use std::fmt::Debug;
use std::str::FromStr;

use anyhow::bail;
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

#[derive(Clone)]
pub struct AString<const L: usize>([u8; L]);

impl<const L: usize> AString<L> {
    pub fn to_string(&self) -> String {
        let s = self.0.as_slice().split(|b| *b == 0).next().unwrap();
        String::from_utf8(s.to_owned()).unwrap()
    }
}

impl<'a, const L: usize> DekuRead<'a> for AString<L> {
    fn read(
        input: &'a BitSlice<u8, Msb0>,
        ctx: (),
    ) -> Result<(&'a BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (rest, array): (&'a BitSlice<u8, Msb0>, [u8; L]) = DekuRead::read(input, ctx)?;
        let s = array.as_slice().split(|b| *b == 0).next().unwrap();
        match std::str::from_utf8(s) {
            Ok(_) => {
                // this is valid UTF-8 so we'll return it
                Ok((rest, AString(array)))
            }
            Err(err) => {
                // not valid UTF-8, raise the alarm
                Err(DekuError::Parse(format!("Invalid String: {err}")))
            }
        }
    }
}

impl<const L: usize> DekuWrite for AString<L> {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        self.0.write(output, ctx)
    }
}

impl<const L: usize> Debug for AString<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.0.as_slice().split(|b| *b == 0).next().unwrap();
        let s = std::str::from_utf8(s).unwrap();
        let s = s.trim_end_matches('\0');
        s.fmt(f)
    }
}

impl<const L: usize> FromStr for AString<L> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() >= L {
            bail!("string too long")
        }

        let mut result = Self([0; L]);
        result.0[0..s.len()].copy_from_slice(s.as_bytes());
        Ok(result)
    }
}

#[derive(Clone)]
pub struct WString<const L: usize>([u16; L]);

impl<const L: usize> Default for WString<L> {
    fn default() -> Self {
        WString([0; L])
    }
}

impl<const L: usize> WString<L> {
    pub fn to_string(&self) -> String {
        let s = self.0.as_slice().split(|b| *b == 0).next().unwrap();
        String::from_utf16_lossy(s)
    }
}

impl<'a, const L: usize> DekuRead<'a> for WString<L> {
    fn read(
        input: &'a BitSlice<u8, Msb0>,
        ctx: (),
    ) -> Result<(&'a BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (rest, array): (&'a BitSlice<u8, Msb0>, [u16; L]) = DekuRead::read(input, ctx)?;
        Ok((rest, WString(array)))
    }
}

impl<const L: usize> DekuWrite for WString<L> {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        self.0.write(output, ctx)
    }
}

impl<const L: usize> Debug for WString<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::from_utf16_lossy(&self.0);
        let s = s.trim_end_matches('\0');
        s.fmt(f)
    }
}

impl<const L: usize> FromStr for WString<L> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Self([0; L]);

        for (i, ch) in s.encode_utf16().enumerate() {
            if i >= L {
                bail!("string too long")
            }
            result.0[i] = ch;
        }

        Ok(result)
    }
}
