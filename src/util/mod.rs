use std::convert::TryFrom;
use crate::protocol::{structs::{Readable, ProtocolVersion, Writeable}};

/// An angle written so that 1.0 = 1/256th of a turn
pub struct Angle(pub f32);

//Should this be a &str? 
pub struct Identifier(String);

impl Identifier {
    pub fn new(namespace: String, value: String) -> Self {
        Identifier((namespace + ":" + &value).into())
    }
}

impl Readable for Identifier {
    fn read(buffer: &mut std::io::Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        Identifier::try_from(String::read(buffer, version)?)
    }
}

impl Writeable for Identifier {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let inner = &self.0;
        inner.write(buffer, version)
    }
}

impl TryFrom<String> for Identifier {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut parts: Vec<&str> = Vec::new();
        value.split(":").for_each(|s|parts.push(s));

        if parts.len() != 2 {
            Err(anyhow::anyhow!("Expected two parts separated by colon, found {} parts: {:?}", parts.len(), parts))
        } else {
            Result::Ok(Identifier::new(parts[0].to_owned(), parts[1].to_owned()))
        }
    }
}

impl From<Angle> for f32 {
    fn from(angle: Angle) -> Self {
        angle.0
    }
}

impl Readable for Angle {
    fn read(buffer: &mut std::io::Cursor<&[u8]>, version: crate::protocol::structs::ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized 
    {
        let val = u8::read(buffer, version)?;
        Ok(Angle((val as f32 / 256.0) * 360.0))
    }
}

impl Writeable for Angle {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let temp = (256.0 / 360.0) * (self.0 * 360.0);

        let val = ((temp + 256.0) % 256.0) as u8;
        val.write(buffer, version)?;

        Ok(())
    }
}