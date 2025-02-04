use image::ImageFormat;
use nom::{
    bytes::complete::tag,
    multi::{length_data, length_value},
    number::complete::{be_u32, be_u8},
    IResult,
};

use crate::model::{Model, ModelTexture};

fn parse_texture(i: &[u8]) -> IResult<&[u8], ModelTexture> {
    let (i, format) = be_u8(i)?;
    let format = match format {
        0 => ImageFormat::Png,
        1 => ImageFormat::Tga,
        2 => todo!("Unsupported format BC7"),
        _ => todo!("Unknown format {format}!"),
    };
    let data = i.to_vec();
    Ok((b"", ModelTexture { format, data }))
}

/// Trans rights!
const MAGIC: &[u8] = b"TRNSRTS\0";
/// Text section header
const TEX: &[u8] = b"TEX_SECT";
/// Extended section header
// const EXT: &[u8] = b"EXT_SECT";

fn be_u32_plus_1(i: &[u8]) -> IResult<&[u8], u32> {
    let (i, int) = be_u32(i)?;
    Ok((i, 1 + int))
}

pub fn parse_inp(i: &[u8]) -> IResult<&[u8], Model> {
    // TODO: proper error handling with nom? Currently really weird.
    let (i, _) = tag(MAGIC)(i)?;
    let (i, json) = length_data(be_u32)(i)?;

    let (i, _) = tag(TEX)(i)?;
    let (mut i, num_textures) = be_u32(i)?;
    let mut textures = Vec::new();
    for _ in 0..num_textures {
        let (i2, texture) = length_value(be_u32_plus_1, parse_texture)(i)?;
        textures.push(texture);
        i = i2;
    }

    let puppet = match serde_json::from_slice(json) {
        Ok(puppet) => puppet,
        Err(a) => {
            panic!("{}", a);
        }
    };
    Ok((i, Model { puppet, textures }))
}
