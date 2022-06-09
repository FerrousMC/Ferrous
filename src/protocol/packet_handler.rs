use crate::{VarInt, ProtocolVersion, Readable, Writeable};
use aes::Aes128;
use aes::cipher::{AsyncStreamCipher, KeyIvInit};
use bytes::BytesMut;
use cfb8::{Encryptor, Decryptor};
use flate2::{
    bufread::{ZlibDecoder, ZlibEncoder},
    Compression,
};
use std::io::{Cursor, Read};

pub type EncryptionKey = [u8; 16];
pub type CompressionThreshold = usize;

pub type Cfb8Enc = cfb8::Encryptor<Aes128>;
pub type Cfb8Dec = cfb8::Decryptor<Aes128>;

pub const PROTOCOL: ProtocolVersion = ProtocolVersion::V1_18_2;

//NOTE: A lot of this will be flagged by the compiler as "unused" since the netty thread isn't implemeneted yet.

pub struct EncryptionHandler {
    key: EncryptionKey,
    encryptor: Cfb8Enc,
    decryption: Cfb8Dec,
}

impl EncryptionHandler {
    pub fn new(key: EncryptionKey) -> Self {
        EncryptionHandler { 
            key: key, 
            encryptor: Cfb8Enc::new_from_slices(&key, &key)
                .expect("invalid key size!"), 
            decryption: Cfb8Dec::new_from_slices(&key, &key)
                .expect("invalid key size!") 
        }
    }
}

#[derive(Default)]
pub struct PacketHandler {
    //Encryption stuff
    encryption_handler: Option<EncryptionHandler>,
    //Compression settings
    compression: Option<CompressionThreshold>,

    //Buffers
    incoming_buf: BytesMut,
    writing_buf: Vec<u8>,
    compressed_buf: Vec<u8>
}

impl PacketHandler {
    fn new() -> Self {
        Self::default()
    }

    pub fn clone_keep_settings(&self) -> Self {
        PacketHandler {
            encryption_handler: self.encryption_handler.as_ref().map(|v|EncryptionHandler::new(v.key)),
            ..Default::default()
        }
    }

    pub fn enable_encryption(&mut self, key: EncryptionKey) {
        self.encryption_handler = Some(EncryptionHandler::new(key))
    }

    //this function could probably be more readable
    pub fn next_packet<T>(&mut self) -> anyhow::Result<Option<T>>
    where
        T: Readable,
    {
        let mut cursor = Cursor::new(&self.incoming_buf[..]);
        let length: i32;

        if let Ok(len) = VarInt::read(&mut cursor, PROTOCOL) {
            length = len.0;
        } else {
            return Err(anyhow::anyhow!("unable to read packet."));
        }

        //the length (in bytes) of the VarInt that describes length
        let length_field_len: usize = cursor.position() as usize;

        return if self.incoming_buf.len() - length_field_len >= length as usize {
            cursor = Cursor::new(
                                         // starting after    // ending with the 
                                         // length encoding   // current section
                &self.incoming_buf[length_field_len .. length_field_len + length as usize]
            );

            if self.compression.is_some() {
                let data_length: i32 = VarInt::read(&mut cursor, PROTOCOL)?.0;
                if data_length != 0 {
                    let mut decompressor = ZlibDecoder::new(&cursor.get_ref()[cursor.position() as usize.. ]);
                    decompressor.read_to_end(&mut self.compressed_buf);
                    cursor = Cursor::new(&self.compressed_buf);
                }
            }

            let packet = T::read(&mut cursor, PROTOCOL)?;
            let bytes_read = length as usize + length_field_len;
            self.incoming_buf = self.incoming_buf.split_off(bytes_read);

            self.compressed_buf.clear();
            Ok(Some(packet))
        }else {
            Err(anyhow::anyhow!("packet had invalid length."))
        }
    }
}