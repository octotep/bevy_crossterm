use bevy::prelude::*;

use bevy_crossterm::prelude::*;

use Color::*;
static COLORS: &[Color] = &[
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
];

pub fn setup(mut commands: Commands, scene_root: Res<Entity>, window: Res<CrosstermWindow>, mut sprites: ResMut<Assets<Sprite>>, mut stylemaps: ResMut<Assets<StyleMap>>) {
    const Y_MARGIN: i32 = 2;

    let title_sprite = Sprite::new("bevy_crossterm supports up to 24-bit color! (on terminals where supported)");
    let title_pos = Position::with_xy(window.x_center() as i32 - title_sprite.x_center() as i32, Y_MARGIN);

    let default_style = stylemaps.add(StyleMap::default());

    commands
        .spawn_bundle(SpriteBundle {
            sprite: sprites.add(title_sprite),
            position: title_pos,
            stylemap: default_style,
            ..Default::default()
        })
        .insert(Parent(*scene_root));

    let space = sprites.add(Sprite::new("    "));

    // Normal terminal colors
    let color_x_start = window.x_center() as i32 - 2 * 16;
    for (num, color) in COLORS.iter().enumerate() {
        let position = Position::with_xy(color_x_start + (num as i32 * 4), Y_MARGIN * 2);

        let stylemap = stylemaps.add(StyleMap::with_bg(*color));

        commands
            .spawn_bundle(SpriteBundle {
                sprite: space.clone(),
                stylemap,
                position,
                ..Default::default()
            })
            .insert(Parent(*scene_root));
    }

    // ANSI colors
    let smaller_space = sprites.add(Sprite::new("   "));
    let colors_per_row = (window.width() as i32 - 4) / 3;
    let color_y_start = Y_MARGIN * 3;
    let color_x_start = 2;
    for color in 0u8..=255 {
        let stylemap = stylemaps.add(StyleMap::with_bg(Color::AnsiValue(color)));

        let linenum = color as i32 / colors_per_row;
        let offset = color as i32 % colors_per_row;
        let position = Position::with_xy(color_x_start + (offset * 3), color_y_start + linenum);

        commands
            .spawn_bundle(SpriteBundle {
                sprite: smaller_space.clone(),
                stylemap,
                position,
                ..Default::default()
            })
            .insert(Parent(*scene_root));
    }

    // Linear gradients for RGB colors

    let smallest_space = sprites.add(Sprite::new("  "));
    let red_y_start = color_y_start + (256 / colors_per_row) + Y_MARGIN;
    let gradient_per_row = (window.width() as usize - 4) / 2;
    let color_step = 255 / gradient_per_row as u8;
    // Red
    for red in 0..=gradient_per_row {
        let color = Color::Rgb {
            r: (color_step * red as u8),
            b: 0,
            g: 0,
        };

        let position = Position::with_xy(color_x_start + (red as i32 * 2), red_y_start);

        let stylemap = stylemaps.add(StyleMap::with_bg(color));

        commands
            .spawn_bundle(SpriteBundle {
                sprite: smallest_space.clone(),
                position,
                stylemap,
                ..Default::default()
            })
            .insert(Parent(*scene_root));
    }

    // Blue
    for blue in 0..=gradient_per_row {
        let color = Color::Rgb {
            r: 0,
            b: (color_step * blue as u8),
            g: 0,
        };

        let position = Position::with_xy(color_x_start + (blue as i32 * 2), red_y_start + Y_MARGIN);

        let stylemap = stylemaps.add(StyleMap::with_bg(color));

        commands
            .spawn_bundle(SpriteBundle {
                sprite: smallest_space.clone(),
                position,
                stylemap,
                ..Default::default()
            })
            .insert(Parent(*scene_root));
    }

    // Green
    for green in 0..=gradient_per_row {
        let color = Color::Rgb {
            r: 0,
            b: 0,
            g: (color_step * green as u8),
        };

        let position = Position::with_xy(color_x_start + (green as i32 * 2), red_y_start + Y_MARGIN * 2);

        let stylemap = stylemaps.add(StyleMap::with_bg(color));

        commands
            .spawn_bundle(SpriteBundle {
                sprite: smallest_space.clone(),
                position,
                stylemap,
                ..Default::default()
            })
            .insert(Parent(*scene_root));
    }

    // All
    for code in 0..=gradient_per_row {
        let color = Color::Rgb {
            r: (color_step * code as u8),
            b: (color_step * code as u8),
            g: (color_step * code as u8),
        };

        let position = Position::with_xy(color_x_start + (code as i32 * 2), red_y_start + Y_MARGIN * 3);

        let stylemap = stylemaps.add(StyleMap::with_bg(color));

        commands
            .spawn_bundle(SpriteBundle {
                sprite: smallest_space.clone(),
                position,
                stylemap,
                ..Default::default()
            })
            .insert(Parent(*scene_root));
    }
}
