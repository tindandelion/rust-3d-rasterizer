//! Kitty graphics protocol terminal: alternate screen, raw mode, centered RGB display.

use std::io::{self, Write};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use crossterm::{
    ExecutableCommand,
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct KittyTerminal {
    stdout: io::Stdout,
}

impl KittyTerminal {
    /// Kitty payloads are base64-encoded; each APC chunk must be ≤ 4096 bytes (spec limit).
    const CHUNK_BYTES: usize = 4096;

    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }

    pub fn enter(&mut self) -> io::Result<()> {
        // Raw mode: read keys immediately without waiting for Enter and without echoing them.
        terminal::enable_raw_mode()?;

        // Alternate screen + hidden cursor give a clean full-screen canvas for the demo.
        self.stdout.execute(EnterAlternateScreen)?.execute(Hide)?;
        Ok(())
    }

    pub fn leave(&mut self) -> io::Result<()> {
        // Always restore terminal state, even if the caller returned an error.
        let _ = self.delete_all();
        self.stdout.execute(Show)?.execute(LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// Displays an RGB buffer centered in the terminal window.
    ///
    /// Wire format: `ESC _ G <key=value,...> ; <base64 payload> ESC \`
    ///
    /// Kitty anchors images at the text cursor. This method converts the centered
    /// pixel position to a cursor cell plus `X`/`Y` sub-cell offsets, then sends:
    /// - `a=T` — transmit and display in one step
    /// - `f=24` — 24-bit RGB (3 bytes per pixel)
    /// - `s`, `v` — image width and height in pixels
    /// - `X`, `Y` — sub-cell pixel offset within the cursor cell
    /// - `C=1` — do not move the text cursor after placement
    /// - `i=1` — image id (for later deletion)
    /// - `q=1` — suppress terminal acknowledgement responses on stdin
    /// - `m` — multipart chunk flag (`1` = more coming, `0` = last chunk)
    pub fn display_rgb(
        &mut self,
        pixels: impl AsRef<[u8]>,
        width: u32,
        height: u32,
    ) -> io::Result<()> {
        let window = terminal::window_size()?;
        let (screen_w, screen_h) = self.pixel_dimensions()?;

        let top_left_x = (screen_w.saturating_sub(width)) / 2;
        let top_left_y = (screen_h.saturating_sub(height)) / 2;

        let cell_w = (screen_w / window.columns.max(1) as u32).max(1);
        let cell_h = (screen_h / window.rows.max(1) as u32).max(1);
        let col = top_left_x / cell_w;
        let row = top_left_y / cell_h;
        let x_offset = top_left_x % cell_w;
        let y_offset = top_left_y % cell_h;

        self.stdout.execute(MoveTo(col as u16, row as u16))?;

        let encoded = STANDARD.encode(pixels);
        let chunks: Vec<&[u8]> = encoded.as_bytes().chunks(Self::CHUNK_BYTES).collect();
        for (index, chunk) in chunks.iter().enumerate() {
            write!(self.stdout, "\x1b_G")?;
            if index == 0 {
                write!(
                    self.stdout,
                    "a=T,f=24,s={width},v={height},X={x_offset},Y={y_offset},C=1,i=1,q=1"
                )?;
            }
            if chunks.len() > 1 {
                let more = u8::from(index + 1 < chunks.len());
                write!(self.stdout, "{}m={more}", if index == 0 { "," } else { "" },)?;
            } else {
                write!(self.stdout, ",m=0")?;
            }
            write!(self.stdout, ";")?;
            self.stdout.write_all(chunk)?;
            write!(self.stdout, "\x1b\\")?;
        }
        self.stdout.flush()
    }

    pub fn wait_for_key(&mut self) -> io::Result<()> {
        loop {
            if event::poll(std::time::Duration::from_secs(3600))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => break,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn pixel_dimensions(&self) -> io::Result<(u32, u32)> {
        let window = terminal::window_size()?;
        if window.width > 0 && window.height > 0 {
            return Ok((window.width as u32, window.height as u32));
        }

        // Some terminals leave pixel fields at zero; approximate from cell count.
        Ok((window.columns as u32 * 8, window.rows as u32 * 16))
    }

    fn delete_all(&mut self) -> io::Result<()> {
        write!(&self.stdout, "\x1b_Ga=d,q=1;\x1b\\")?;
        self.stdout.flush()
    }
}
