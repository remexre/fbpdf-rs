use failure::Fallible;
use framebuffer::Framebuffer;
use std::path::Path;

/// A wrapper that encapsulates a framebuffer.
#[derive(Debug)]
pub struct Screen {
    fb: Framebuffer,
}

impl Screen {
    /// Creates a new framebuffer from `/dev/fb0`.
    pub fn new() -> Fallible<Screen> {
        Screen::new_with_path("/dev/fb0")
    }

    /// Creates a new framebuffer from the given path.
    pub fn new_with_path(path: impl AsRef<Path>) -> Fallible<Screen> {
        let fb = Framebuffer::new(path)?;
        Ok(Screen { fb })
    }

    /// Returns the dimensions of the screen.
    pub fn dims(&self) -> (usize, usize) {
        let height = self.fb.var_screen_info.yres;
        let width = self.fb.var_screen_info.xres;
        (width as usize, height as usize)
    }

    /// Draws the given buffer to the screen. The buffer must be the appropriate size.
    pub fn draw_bgr_buf(&mut self, buf: &[u8]) {
        self.fb.write_frame(&buf);
    }

    /// Draws using the given function to plot pixels. The function should return an `(r, g, b)`
    /// tuple.
    pub fn draw_with_fn(&mut self, mut func: impl FnMut(usize, usize) -> (u8, u8, u8)) {
        let bytes_per_px = (self.fb.var_screen_info.bits_per_pixel / 8) as usize;
        let line_length = self.fb.fix_screen_info.line_length as usize;
        let height = self.fb.var_screen_info.yres as usize;

        let mut buf = vec![0u8; line_length * height];
        for (y, line) in buf.chunks_mut(line_length).enumerate() {
            for (x, p) in line.chunks_mut(bytes_per_px).enumerate() {
                let (r, g, b) = func(x, y);
                p[0] = b;
                p[1] = g;
                p[2] = r;
            }
        }

        self.draw_bgr_buf(&buf);
    }
}
