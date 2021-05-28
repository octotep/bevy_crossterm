use bevy::prelude::*;
use bevy_crossterm::prelude::*;

pub fn setup(
    mut commands: Commands,
    scene_root: Res<Entity>,
    window: Res<CrosstermWindow>,
    asset_server: Res<AssetServer>,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    const MARGIN: i32 = 4;

    let mut explain_text = "Sprites are the lifeblood of bevy_crossterm.\n\nSprites are rectangular regions of (unicode) text which are rendered on the screen at a given Position. Sprites can also be visible/invisible and \"transparent\" which allows bevy_crossterm to skip rendering unstyled spaces, so the sprites underneath are visible.\n\nPositions have an x, y, and z coordinate to allow them to be properly ordered front-to-back.".to_string();
    let explain_width = window.x_center() as i32 - MARGIN * 2; // Half the screen plus some margins
    textwrap::fill_inplace(&mut explain_text, explain_width as usize);
    let explain_text = explain_text.replace("\n", "\n\n");
    let explain_sprite = Sprite::new(explain_text);
    let explain_pos = Position::with_xy(
        MARGIN,
        window.y_center() as i32 - explain_sprite.y_center() as i32,
    );

    let default_style = stylemaps.add(StyleMap::default());

    let big_box_handle: Handle<Sprite> = asset_server.get_handle("demo/big_box.txt");
    let big_box_sprite = sprites.get(&big_box_handle).unwrap();
    let big_box_pos = Position::with_xy(
        window.width() as i32 / 4 * 3 - big_box_sprite.width() as i32 - MARGIN,
        window.height() as i32 / 10 * 1,
    );

    let small_box_handle: Handle<Sprite> = asset_server.get_handle("demo/small_box.txt");
    let small_box_pos = Position::with_xy(
        window.width() as i32 / 4 * 3 + MARGIN,
        window.height() as i32 / 10 * 1 + 1,
    );

    let big_combo_pos = Position::with_xy(
        (window.width() as i32 / 4 * 3) - (big_box_sprite.x_center() as i32),
        (window.height() as i32 / 10 * 5) - (big_box_sprite.y_center() as i32),
    );
    let small_combo_pos = Position::new(
        (window.width() as i32 / 4 * 3) - (big_box_sprite.x_center() as i32) + 1,
        (window.height() as i32 / 10 * 5) - (big_box_sprite.y_center() as i32) + 1,
        1,
    );

    let big_combo_trans_pos = Position::with_xy(
        (window.width() as i32 / 4 * 3) - (big_box_sprite.x_center() as i32),
        (window.height() as i32 / 10 * 8) - (big_box_sprite.y_center() as i32),
    );
    let small_combo_trans_pos = Position::new(
        (window.width() as i32 / 4 * 3) - (big_box_sprite.x_center() as i32) + 1,
        (window.height() as i32 / 10 * 8) - (big_box_sprite.y_center() as i32) + 1,
        1,
    );

    let mut white = StyleMap::with_fg(Color::White);
    white.style.attributes.set(Attribute::Bold);
    let white_handle = stylemaps.add(white);
    let grey_handle = stylemaps.add(StyleMap::with_fg(Color::DarkGrey));
    let transparent = Visible::transparent();

    commands
        .spawn_bundle(SpriteBundle {
            sprite: sprites.add(explain_sprite),
            stylemap: default_style.clone(),
            position: explain_pos,
            ..Default::default()
        })
        .insert(Parent(*scene_root));
    // These are the two boxes separately
    commands
        .spawn_bundle(SpriteBundle {
            sprite: asset_server.get_handle("demo/big_box.txt"),
            position: big_box_pos,
            stylemap: grey_handle.clone(),
            ..Default::default()
        })
        .insert(Parent(*scene_root));
    commands
        .spawn_bundle(SpriteBundle {
            sprite: asset_server.get_handle("demo/small_box.txt"),
            stylemap: white_handle.clone(),
            position: small_box_pos,
            ..Default::default()
        })
        .insert(Parent(*scene_root));
    // These are the sprites that make up the non-transparent demo
    commands
        .spawn_bundle(SpriteBundle {
            sprite: big_box_handle.clone(),
            position: big_combo_pos,
            stylemap: grey_handle.clone(),
            ..Default::default()
        })
        .insert(Parent(*scene_root));
    commands
        .spawn_bundle(SpriteBundle {
            sprite: small_box_handle.clone(),
            position: small_combo_pos,
            stylemap: white_handle.clone(),
            ..Default::default()
        })
        .insert(Parent(*scene_root));
    // These are the sprites that make up the trasparent demo
    commands
        .spawn_bundle(SpriteBundle {
            sprite: big_box_handle.clone(),
            position: big_combo_trans_pos,
            stylemap: grey_handle.clone(),
            ..Default::default()
        })
        .insert(Parent(*scene_root));
    commands
        .spawn_bundle(SpriteBundle {
            sprite: small_box_handle.clone(),
            position: small_combo_trans_pos,
            stylemap: white_handle.clone(),
            visible: transparent,
        })
        .insert(Parent(*scene_root));
}
