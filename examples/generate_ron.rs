use bevy_crossterm::prelude::*;

// This doesn't really demonstrate anything, but it does show how to save a stylemap as an asset.

fn main() {
    let mut stylemap = StyleMap::default();
    let w = Style::with_colors(Colors::new(Color::White, Color::Black));
    let r = Style::with_colors(Colors::new(Color::White, Color::AnsiValue(160)));
    let o = Style::with_colors(Colors::new(Color::White, Color::AnsiValue(166)));
    let y = Style::with_colors(Colors::new(Color::White, Color::AnsiValue(178)));
    let g = Style::with_colors(Colors::new(Color::White, Color::AnsiValue(34)));
    let b = Style::with_colors(Colors::new(Color::White, Color::AnsiValue(27)));
    let i = Style::with_colors(Colors::new(Color::White, Color::AnsiValue(19)));
    let v = Style::with_colors(Colors::new(Color::White, Color::AnsiValue(91)));
    stylemap
        .map
        .push(vec![w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w]);
    stylemap
        .map
        .push(vec![w, w, r, o, y, g, b, i, v, r, o, y, g, b, i, v, w, w]);
    stylemap
        .map
        .push(vec![w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w]);

    let file = std::fs::File::create("bounce.stylemap").unwrap();
    ron::ser::to_writer(&file, &stylemap).unwrap();
}
