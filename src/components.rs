use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::default::Default;
use unicode_segmentation::UnicodeSegmentation;

pub use crossterm::style::Color;

#[derive(Default)]
pub(crate) struct PreviousEntityDetails(pub HashMap<Entity, (PreviousPosition, PreviousSize)>);

pub(crate) struct PreviousWindowColors(pub Colors);

impl Default for PreviousWindowColors {
    fn default() -> Self {
        PreviousWindowColors(Colors::term_colors())
    }
}

#[derive(Default)]
pub(crate) struct EntitiesToRedraw {
    pub full_redraw: bool,
    pub to_clear: HashSet<Entity>,
    pub to_draw: Vec<EntityDepth>,
}

pub(crate) struct EntityDepth {
    pub entity: Entity,
    pub z: i32,
}

#[derive(Bundle, Default)]
pub struct SpriteBundle {
    pub sprite: Handle<Sprite>,
    pub position: Position,
    pub stylemap: Handle<StyleMap>,
    pub visible: Visible,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Colors {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}
impl Default for Colors {
    fn default() -> Self {
        Colors {
            foreground: None,
            background: None,
        }
    }
}

impl Colors {
    pub fn term_colors() -> Colors {
        Colors {
            foreground: Some(Color::Reset),
            background: Some(Color::Reset),
        }
    }

    // Returns the color which represents either this struct, or provided defaults if this struct is empty
    pub fn with_default(&self, default_colors: Colors) -> Colors {
        Colors {
            foreground: self.foreground.or(default_colors.foreground),
            background: self.background.or(default_colors.background),
        }
    }

    pub fn new(foreground: Color, background: Color) -> Colors {
        Colors {
            foreground: Some(foreground),
            background: Some(background),
        }
    }

    pub fn bg(background: Color) -> Colors {
        Colors {
            foreground: None,
            background: Some(background),
        }
    }

    pub fn fg(foreground: Color) -> Colors {
        Colors {
            foreground: Some(foreground),
            background: None,
        }
    }

    pub fn to_crossterm(&self) -> crossterm::style::Colors {
        crossterm::style::Colors {
            foreground: self.foreground,
            background: self.background,
        }
    }
}

mod attribute_parser {
    use serde::de::Visitor;
    use serde::{Deserializer, Serializer};
    pub fn serialize<S>(
        attrs: &crossterm::style::Attributes,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut attr_bits = 0u32;
        for attr in crossterm::style::Attribute::iterator() {
            if attrs.has(attr) {
                attr_bits |= attr.bytes();
            }
        }
        serializer.serialize_u32(attr_bits)
    }

    struct AttrVisitor;

    impl<'de> Visitor<'de> for AttrVisitor {
        type Value = crossterm::style::Attributes;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("expecting an u32")
        }

        fn visit_u32<E>(self, attr_bits: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut attrs = crossterm::style::Attributes::default();
            for attr in crossterm::style::Attribute::iterator() {
                if attr_bits.clone() & attr.bytes() != 0 {
                    attrs.set(attr);
                }
            }
            Ok(attrs)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<crossterm::style::Attributes, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(AttrVisitor)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Style {
    #[serde(with = "attribute_parser")]
    pub attributes: crossterm::style::Attributes,
    pub colors: Colors,
}

impl Style {
    pub fn new(colors: Colors, attributes: crossterm::style::Attributes) -> Style {
        Style { colors, attributes }
    }

    pub fn with_attrib(attribute: crossterm::style::Attribute) -> Style {
        Style {
            colors: Colors::default(),
            attributes: attribute.into(),
        }
    }

    pub fn with_attribs(attributes: crossterm::style::Attributes) -> Style {
        Style {
            colors: Colors::default(),
            attributes,
        }
    }

    pub fn with_fg(foreground: Color) -> Style {
        Style {
            colors: Colors::fg(foreground),
            attributes: crossterm::style::Attribute::Reset.into(),
        }
    }

    pub fn with_bg(background: Color) -> Style {
        Style {
            colors: Colors::bg(background),
            attributes: crossterm::style::Attribute::Reset.into(),
        }
    }

    pub fn with_colors(colors: Colors) -> Style {
        Style {
            colors,
            attributes: crossterm::style::Attribute::Reset.into(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style {
            attributes: crossterm::style::Attribute::Reset.into(),
            colors: Colors::default(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, TypeUuid)]
#[uuid = "a5418d12-e050-498a-a31e-37fd0b6c078d"]
pub struct StyleMap {
    pub style: Style,
    pub map: Vec<Vec<Style>>,
}

impl StyleMap {
    pub fn new(style: Style, map: Vec<Vec<Style>>) -> StyleMap {
        StyleMap { style, map }
    }

    pub fn with_attrib(attribute: crossterm::style::Attribute) -> StyleMap {
        StyleMap {
            style: Style::with_attrib(attribute),
            ..Default::default()
        }
    }

    pub fn with_attribs(attributes: crossterm::style::Attributes) -> StyleMap {
        StyleMap {
            style: Style::with_attribs(attributes),
            ..Default::default()
        }
    }

    pub fn with_fg(foreground: Color) -> StyleMap {
        StyleMap {
            style: Style::with_fg(foreground),
            ..Default::default()
        }
    }

    pub fn with_bg(background: Color) -> StyleMap {
        StyleMap {
            style: Style::with_bg(background),
            ..Default::default()
        }
    }

    pub fn with_colors(colors: Colors) -> StyleMap {
        StyleMap {
            style: Style::new(colors, crossterm::style::Attributes::default()),
            ..Default::default()
        }
    }

    /// If there is a style available in the map, this fetches it. Otherwise, this returns None
    pub fn style_at(&self, x: usize, y: usize) -> Option<&Style> {
        self.map.get(y).and_then(|vec| vec.get(x))
    }

    /// If there is a style for the grapheme at position x,y in the map, this fetches it. Otherwise
    /// the global sprite's style is returned
    pub fn style_for(&self, x: usize, y: usize) -> Style {
        let grapheme = self.style_at(x, y);
        if let Some(style) = grapheme {
            *style
        } else {
            self.style
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Visible {
    pub is_visible: bool,
    pub is_transparent: bool,
}

impl Default for Visible {
    fn default() -> Self {
        Visible {
            is_visible: true,
            is_transparent: false,
        }
    }
}

impl Visible {
    pub fn invisible() -> Visible {
        Visible {
            is_transparent: false,
            is_visible: false,
        }
    }

    pub fn transparent() -> Visible {
        Visible {
            is_visible: true,
            is_transparent: true,
        }
    }
}

#[derive(Default, Eq, PartialEq, Debug, TypeUuid)]
#[uuid = "f04f5352-e656-4a90-95a5-2269c02d0091"]
pub struct Sprite {
    // The whole sprites's data
    data: String,
    // Each tuple represents a unicode grapheme. This allows us to know where each
    // whole character is easily. Since these are indices into the data field, they
    // must be updated in tandem
    graphemes: Vec<Vec<(usize, usize)>>,
    max_width: usize,
}

impl Sprite {
    pub fn new<T: std::string::ToString>(value: T) -> Sprite {
        let mut sprite = Sprite::default();
        sprite.data = value.to_string();

        Sprite::convert_to_sprite(&mut sprite);

        sprite
    }

    fn convert_to_sprite(sprite: &mut Sprite) {
        sprite.max_width = 0;

        let mut current_line = Vec::new();
        for (start, grapheme) in UnicodeSegmentation::grapheme_indices(&*sprite.data, true) {
            if grapheme == "\r" || grapheme == "\n" || grapheme == "\r\n" {
                sprite.max_width = std::cmp::max(sprite.max_width, current_line.len());
                sprite.graphemes.push(std::mem::take(&mut current_line));
                continue;
            }

            current_line.push((start, start + grapheme.len()));
        }

        if !current_line.is_empty() {
            sprite.max_width = std::cmp::max(sprite.max_width, current_line.len());
            sprite.graphemes.push(std::mem::take(&mut current_line));
        }
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn width(&self) -> usize {
        self.max_width
    }

    pub fn height(&self) -> usize {
        self.graphemes.len()
    }

    pub fn x_center(&self) -> usize {
        self.width() / 2
    }

    pub fn y_center(&self) -> usize {
        self.height() / 2
    }

    pub fn graphemes(&self) -> &[Vec<(usize, usize)>] {
        &self.graphemes
    }

    pub fn grapheme(&self, grapheme: &(usize, usize)) -> &str {
        &self.data[grapheme.0..grapheme.1]
    }

    pub fn update<T: std::string::ToString>(&mut self, value: T) {
        self.data = value.to_string();
        self.graphemes.clear();
        Sprite::convert_to_sprite(self);
    }
}

#[derive(Default, Eq, PartialEq, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Position {
        Position { x, y, z }
    }

    pub fn with_x(x: i32) -> Position {
        Position {
            x,
            ..Default::default()
        }
    }

    pub fn with_y(y: i32) -> Position {
        Position {
            y,
            ..Default::default()
        }
    }

    pub fn with_xy(x: i32, y: i32) -> Position {
        Position {
            x,
            y,
            ..Default::default()
        }
    }
}

#[derive(Default, Eq, PartialEq, Debug)]
pub(crate) struct PreviousPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Default, Eq, PartialEq, Debug)]
pub(crate) struct PreviousSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Default, Eq, PartialEq, Debug)]
pub(crate) struct GlobalPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
