
// use crate::{ProtocolVersion, Slot};
use anyhow::{anyhow, bail, Context, Ok};
// use base::{
//     anvil::entity::ItemNbt, metadata::MetaEntry, BlockId, BlockPosition, Direction, EntityMetadata,
//     Gamemode, Item, ItemStackBuilder, ValidBlockPosition,
// };
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
// use libcraft_items::InventorySlot::*;
use num_traits::{FromPrimitive, ToPrimitive};
// use quill_common::components::PreviousGamemode;
use serde::{de::DeserializeOwned, Serialize};
use std::char::MAX;
use std::io::ErrorKind;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    io::{self, Cursor, Read, Write},
    iter,
    marker::PhantomData,
    num::TryFromIntError,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProtocolVersion {
    V1_18_2,
}

/// Trait for types which can be read from buffer
pub trait Readable {
    /// Reads this type from the given buffer
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized;
}

/// Trait implemented for types which can be written to a buffer
pub trait Writeable: Sized {
    /// Writes this value to the given buffer
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()>;
}

impl<'a, T> Writeable for &'a T 
where
    T: Writeable,
{
    //idk if dereferencing here is the best idea, but we can always .clone() in the future
    //especially bc cloning would consume and deallocate it
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        T::write(*self, buffer, version)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected end of input: failed to read value of type `{0}`")]
    UnexpectedEof(&'static str)
}

macro_rules! integer_impl {
    ($($int:ty, $read_fn:tt, $write_fn:tt),* $(,)?) => {
        $(
            impl Readable for $int {
                fn read(buffer: &mut Cursor<&[u8]>, _version: ProtocolVersion) -> anyhow::Result<Self> {
                    buffer.$read_fn::<BigEndian>().map_err(anyhow::Error::from)
                }
            }

            impl Writeable for $int {
                fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
                    buffer.$write_fn::<BigEndian>(*self)?;
                    Ok(())
                }
            }
        )*
    }
}

integer_impl! {
    u16, read_u16, write_u16,
    u32, read_u32, write_u32,
    u64, read_u64, write_u64,

    i16, read_i16, write_i16,
    i32, read_i32, write_i32,
    i64, read_i64, write_i64,

    f32, read_f32, write_f32,
    f64, read_f64, write_f64,
}

impl Readable for u8 {
    fn read(buffer: &mut Cursor<&[u8]>, _version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        buffer.read_u8().map_err(anyhow::Error::from)
    }
}

impl Writeable for u8 {
    fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
        buffer.write_u8(*self)?;
        Ok(())
    }
}

impl Readable for i8 {
    fn read(buffer: &mut Cursor<&[u8]>, _version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        buffer.read_i8().map_err(anyhow::Error::from)
    }
}

impl Writeable for i8 {
    fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
        buffer.write_i8(*self)?;
        Ok(())
    }
}

impl Readable for bool {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self> 
    where  
        Self: Sized,
    {
        let x = u8::read(buffer, version)?;
        
        match x {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(anyhow::anyhow!("invalid boolean tag {}", x))
        }
    }   
}

impl Writeable for bool {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let x = if *self {1u8} else {0};
        x.write(buffer, version)?;

        Ok(())
    }
}



impl<T> Readable for Option<T>
where
    T: Readable,
{
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // Assume boolean prefix.
        let present = bool::read(buffer, version)?;

        if present {
            Ok(Some(T::read(buffer, version)?))
        } else {
            Ok(None)
        }
    }
}

impl<T> Writeable for Option<T>
where
    T: Writeable,
{
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let present = self.is_some();
        present.write(buffer, version)?;

        if let Some(value) = self {
            value.write(buffer, version)?;
        }

        Ok(())
    }
}


pub struct VarInt(pub i32);

impl VarInt {
    pub fn write_to(&self, mut writer: impl Write) -> io::Result<usize> {
        let mut x = self.0 as u32;
        let mut i = 0;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }

            writer.write_all(&[temp])?;
            
            i += 1;
            if x == 0 {
                break;
            }
        }
        io::Result::Ok(i)
    }

    pub fn read_from(mut reader: impl Read) -> io::Result<Self> {
        let mut num_read = 0;
        let mut result = 0;

        loop {
            let read = reader.read_u8()?;
            let value = i32::from(read & 0b0111_1111);

            result |= value.overflowing_shl(7 * num_read).0;

            num_read += 1;

            if num_read > 5 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "VarInt too long (max length: 5)",
                ));
            }

            if read & 0b100_000 == 0 {
                break;
            }
        }
        io::Result::Ok(VarInt(result))
    }
}

impl Readable for VarInt {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized 
    {
        Self::read_from(buffer).map_err(Into::into)
    }
}

impl Writeable for VarInt {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        self.write_to(buffer).expect("write to Vec failed");
        Ok(())
    }
}

/// A variable-length i64 as defined in the Minecraft protocol.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VarLong(pub i64);

impl From<VarLong> for i64 {
    fn from(x: VarLong) -> Self {
        x.0
    }
}

impl From<i64> for VarLong {
    fn from(x: i64) -> Self {
        VarLong(x)
    }
}

impl Readable for VarLong {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut num_read = 0;
        let mut result = 0;

        loop {
            let read = u8::read(buffer, version)?;
            let value = i64::from(read & 0b0111_1111);
            result |= value.overflowing_shl(7 * num_read).0;

            num_read += 1;

            if num_read > 10 {
                bail!(
                    "VarInt too long (max length: 5, value read so far: {})",
                    result
                );
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(VarLong(result))
    }
}

impl Writeable for VarLong {
    fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
        let mut x = self.0 as u64;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }

            buffer.write_u8(temp).unwrap();

            if x == 0 {
                break;
            }
        }

        Ok(())
    }
}

impl Readable for String {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized 
    {
        //Length is a VarInt
        //After the "length" bytes everything is utf8

        let length = VarInt::read(buffer, version)
            .context("failed to read string length")?
            .0 as usize;

        let max_length = std::i16::MAX as usize;
        if length > max_length {
            bail!(
                "string length {} exceeds maximum allowed length of {}",
                length,
                max_length
            )
        }

        let mut temp = vec![0u8; length];
        buffer
            .read_exact(&mut temp)
            .map_err(|_| Error::UnexpectedEof("String"))?;

            let s = std::str::from_utf8(&temp).context("String contained invalid UTF8")?;
            Ok(s.to_owned())
    }
}

pub const MAX_LENGTH: usize = 1024 * 1024;

pub struct LengthPrefixedVec<'a, P, T>(pub Cow<'a, [T]>, PhantomData<P>)
where
    [T]: ToOwned<Owned = Vec<T>>;

impl<'a, P, T> Readable for LengthPrefixedVec<'a, P, T>
where
    T: Readable,
    [T]: ToOwned<Owned = Vec<T>>,
    P: TryInto<usize> + Readable,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
        where
            Self: Sized 
    {
        let length: usize = P::read(buffer, version)?.try_into()?;
        
        if length > MAX_LENGTH {
            bail!("array length too large ({} > {})", length, MAX_LENGTH);
        }
        let vec = iter::repeat_with(|| T::read(buffer, version))
            .take(length)
            .collect::<anyhow::Result<Vec<T>>>()?;

        Ok(Self(Cow::Owned(vec), PhantomData))
    }
}

impl<'a, P, T> Writeable for LengthPrefixedVec<'a, P, T> 
where
    T: Writeable,
    [T]: ToOwned<Owned = Vec<T>>,
    P: TryFrom<usize> + Writeable,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()>
    where
        Self: Sized 
    {
        P::try_from(self.0.len())?.write(buffer, version)?;
        self.0
            .iter()
            .for_each(|item| item.write(buffer, version).expect("failed to write to vec"));

        Ok(())
    }
}

impl<'a, P, T> From<LengthPrefixedVec<'a, P, T>> for Vec<T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(x: LengthPrefixedVec<'a, P, T>) -> Self {
        x.0.into_owned()
    }
}

impl<'a, P, T> From<&'a [T]> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(slice: &'a [T]) -> Self {
        Self(Cow::Borrowed(slice), PhantomData)
    }
}

impl<'a, P, T> From<Vec<T>> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(vec: Vec<T>) -> Self {
        Self(Cow::Owned(vec), PhantomData)
    }
}

pub type VarIntPrefixedVec<'a, T> = LengthPrefixedVec<'a, VarInt, T>;
pub type ShortPrefixedVec<'a, T> = LengthPrefixedVec<'a, u16, T>;

pub struct GreedyVecU8<'a>(pub Cow<'a, [u8]>);
impl<'a> Readable for GreedyVecU8<'a> {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized 
    {
        let mut vec = Vec::new();
        buffer.read_to_end(&mut vec)?;
        Ok(GreedyVecU8(Cow::Owned(vec)))
    }
}


impl<'a> Writeable for GreedyVecU8<'a> {
    fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
        buffer.extend_from_slice(&*self.0);
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for GreedyVecU8<'a> {
    fn from(slice: &'a [u8]) -> Self {
        GreedyVecU8(Cow::Borrowed(slice))
    }
}

impl<'a> From<GreedyVecU8<'a>> for Vec<u8> {
    fn from(x: GreedyVecU8<'a>) -> Self {
        x.0.into_owned()
    }
    
}