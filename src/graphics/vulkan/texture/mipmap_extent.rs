use super::MipmapExtent;

impl MipmapExtent {
    /// The expected size of the mipmap based on it's dimensions and the bytes
    /// per pixel.
    pub fn size_in_bytes(&self, bytes_per_pixel: u64) -> u64 {
        (self.width * self.height) as u64 * bytes_per_pixel
    }
}
