mod options;
mod screen;
mod util;

use crate::{options::Options, screen::Screen, util::log_err};
use failure::{format_err, Fallible};
use poppler::PopplerDocument;
use std::io::{stdin, stdout, Read, Stdout};
use structopt::StructOpt;
use termion::raw::{IntoRawMode, RawTerminal};

fn main() {
    let opts = Options::from_args();
    if let Err(err) = run(opts) {
        log_err(err);
        std::process::exit(1);
    }
}

fn run(opts: Options) -> Fallible<()> {
    opts.start_logger()?;

    let mut screen = Screen::new()?;
    let (w, h) = screen.dims();
    let mut surface = cairo::ImageSurface::create(cairo::Format::Rgb24, w as i32, h as i32)
        .map_err(|e| format_err!("Failed to allocate surface: {:?}", e))?;

    let pdf = PopplerDocument::new_from_file(&opts.file, &opts.password)?;
    let n = pdf.get_n_pages();

    let term = stdout().into_raw_mode()?;

    let mut xoff = (h as f64
        - pdf
            .get_page(0)
            .map(|page| page.get_size().0 / 2.0)
            .unwrap_or(0.0))
        / 2.0;
    let mut yoff = 0.0;
    let mut zoom = 1.0;

    'main: loop {
        // Clear the buffer to plain white.
        {
            let mut data = surface.get_data().unwrap();
            for i in 0..(4 * w * h) {
                data[i] = 0xff;
            }
        }

        // Render the PDF.
        let mut ctx = cairo::Context::new(&surface);
        ctx.translate(xoff, yoff);
        ctx.scale(zoom, zoom);
        let mut top = yoff;
        for i in 0..n {
            let page = pdf.get_page(i).unwrap();
            let dy = page.get_size().1;
            if top >= 0.0 || top + dy <= (h as f64) {
                page.render(&mut ctx);
            }
            ctx.translate(0.0, dy);
            top += dy;
            if top >= (h as f64) {
                break;
            }
        }
        drop(ctx);

        // Draw the buffer to the screen.
        screen.draw_bgr_buf(&surface.get_data().unwrap());

        // Handle any user input that's occurred. Multiple keypresses are buffered, since rendering
        // is slow enough that it's entirely possible to type a few keys before the input gets
        // processed.
        let mut keypresses = [0u8; 16];
        for key in get_keypresses(&term, &mut keypresses)? {
            match key {
                3 | 27 | 113 => break 'main,      // c-c, esc, or q
                32 => yoff -= 200.0 * zoom,       // space
                104 | 97 => xoff += 20.0 * zoom,  // h or a
                106 | 115 => yoff -= 20.0 * zoom, // j or s
                108 | 110 => xoff -= 20.0 * zoom, // l or d
                107 | 119 => yoff += 20.0 * zoom, // k or w
                45 | 95 => zoom /= 1.1,           // -
                61 | 43 => zoom *= 1.1,           // +
                key => log::debug!("Unknown key: {:?}", key),
            }
        }
    }

    Ok(())
}

fn get_keypresses<'a>(_term: &RawTerminal<Stdout>, buf: &'a mut [u8]) -> Fallible<&'a [u8]> {
    let n = stdin().read(buf)?;
    Ok(&buf[..n])
}
