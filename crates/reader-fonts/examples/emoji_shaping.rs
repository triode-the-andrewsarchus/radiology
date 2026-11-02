//! Writes a composed monochrome emoji glyph as a portable grayscale image.

use std::{
    error::Error,
    io::{self, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let text = args.next().unwrap_or_else(|| "1\u{fe0f}\u{20e3}".into());
    let height = args
        .next()
        .map_or(Ok(56.0), |height| height.parse::<f32>())?;
    let mut shaper = reader_fonts::EmojiShaper::new();
    let glyph = shaper
        .shape_cluster(&text)
        .ok_or("cluster is not composed by the bundled font")?;
    let image = shaper
        .rasterize(&glyph, height)
        .ok_or("glyph cannot be rasterized at this height")?;
    eprintln!(
        "glyph={} bytes={} scalars={} advance={:?} origin={:?}",
        glyph.glyph_id, glyph.source_bytes, glyph.source_scalars, image.advance, image.origin
    );
    let mut output = io::stdout().lock();
    writeln!(output, "P5\n{} {}\n255", image.width, image.height)?;
    output.write_all(&image.coverage)?;
    Ok(())
}
