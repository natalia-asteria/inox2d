use std::io;
use std::mem::MaybeUninit;

use crate::model::Model;
use crate::texture::CompressedTexture;

use super::serialize::deserialize_puppet;

/// Trans rights!
const MAGIC: &[u8] = b"TRNSRTS\0";

/// Text section header
const TEX: &[u8] = b"TEX_SECT";

/// Extended section header
// const EXT: &[u8] = b"EXT_SECT";

fn read_u8<R: io::Read>(reader: &mut R) -> io::Result<u8> {
    let mut buf = [0_u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_be_u32<R: io::Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0_u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_array<R: io::Read, const N: usize>(reader: &mut R) -> io::Result<[u8; N]> {
    let mut data = [0_u8; N];
    reader.read_exact(&mut data)?;
    Ok(data)
}

fn read_vec<R: io::Read>(reader: &mut R, length: u32) -> io::Result<Vec<u8>> {
    let length = length as usize;
    let mut data: Vec<MaybeUninit<u8>> = Vec::with_capacity(length);
    unsafe { data.set_len(length) };
    let mut data: Vec<u8> = unsafe { std::mem::transmute(data) };
    reader.read_exact(&mut data)?;
    Ok(data)
}

/// Parse a `.inp` Inochi Puppet from memory.
pub fn parse_inp<R: io::Read>(mut reader: R) -> io::Result<Model> {
    let magic = read_array::<R, 8>(&mut reader)?;
    if magic != MAGIC {
        return Err(io::ErrorKind::InvalidData.into());
    }

    let puppet = {
        let length = read_be_u32(&mut reader)?;
        let payload = read_vec(&mut reader, length)?;

        // Hmmm... Is this hacky unchecked thing alright?
        let payload = unsafe { std::str::from_utf8_unchecked(&payload) };
        let payload = json::parse(payload).unwrap_or_else(|e| panic!("Invalid JSON payload: {e}"));
        deserialize_puppet(&payload).unwrap_or_else(|e| panic!("Invalid puppet\n- {e}"))
    };

    let magic = read_array::<R, 8>(&mut reader)?;
    if magic != TEX {
        return Err(io::ErrorKind::InvalidData.into());
    }

    let num_textures = read_be_u32(&mut reader)?;
    let mut textures = Vec::with_capacity(num_textures as usize);
    for _ in 0..num_textures {
        let length = read_be_u32(&mut reader)?;
        let format = read_u8(&mut reader)?;
        let data = read_vec(&mut reader, length)?;
        let texture = match format {
            0 => CompressedTexture::Png(data),
            1 => CompressedTexture::Tga(data),
            2 => CompressedTexture::Bc7(data),
            _ => panic!("Unknown format {format}"),
        };
        textures.push(texture);
    }

    Ok(Model { puppet, textures })
}
