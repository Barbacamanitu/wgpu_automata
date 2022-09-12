use image::{Pixel, Rgba};

pub struct ImageUtil {}

impl ImageUtil {
    pub fn random_image(w: u32, h: u32) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let mut image_buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::new(w, h);
        for (x, y, p) in image_buffer.enumerate_pixels_mut() {
            let black = Rgba::from_slice(&[0, 0, 0, 255]);
            let white = Rgba::from_slice(&[255, 255, 255, 255]);
            if rand::random() {
                *p = *black;
            } else {
                *p = *white;
            }
        }
        image_buffer
    }
}
