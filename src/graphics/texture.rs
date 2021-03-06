extern crate gl;
extern crate image;

use self::image::{DynamicImage, GenericImage};
use self::image::DynamicImage::*;
use std::os::raw::c_void;

pub struct Texture {
    pub id: u32,
}

impl Texture {
    // TODO(orglofch): Decouple this from an image so we can programatically
    // generate textures.
    pub unsafe fn new(image: DynamicImage) -> Texture {
        let mut texture_id = 0;
        gl::GenTextures(1, &mut texture_id);

        let data = image.raw_pixels();
        let format = match image {
            ImageLuma8(_) => gl::RED,
            ImageLumaA8(_) => gl::RG,
            ImageRgb8(_) => gl::RGB,
            ImageRgba8(_) => gl::RGBA,
        };

        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       format as i32,
                       image.width() as i32,
                       image.height() as i32,
                       0,
                       format,
                       gl::UNSIGNED_BYTE,
                       &data[0] as *const u8 as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        Texture {
            id: texture_id
        }
    }
}
