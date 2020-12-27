use bevy::prelude::*;
use bevy_crossterm::prelude::*;

pub fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    window: Res<CrosstermWindow>,
    scene_root: Res<Entity>,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    let title_handle = asset_server.get_handle("demo/title.txt");
    let title_sprite = sprites.get(&title_handle).unwrap();
    let title_pos = Position::with_xy(
        window.x_center() as i32 - title_sprite.x_center() as i32,
        window.height() as i32 / 10 * 1,
    );

    let welcome_sprite = Sprite::new("Welcome to the bevy_crossterm demo!");
    let welcome_pos = Position::with_xy(
        window.x_center() as i32 - welcome_sprite.x_center() as i32,
        window.height() as i32 / 10 * 5,
    );

    let explain_sprite = Sprite::new("In this demo, I will show you different features of bevy_crossterm and how they fit together");
    let explain_pos = Position::with_xy(
        window.x_center() as i32 - explain_sprite.x_center() as i32,
        window.height() as i32 / 10 * 7,
    );

    let press_sprite = Sprite::new("Press any key to advance to the next scene");
    let press_pos = Position::with_xy(
        window.x_center() as i32 - press_sprite.x_center() as i32,
        window.height() as i32 / 10 * 9,
    );

    let color = stylemaps.add(StyleMap::default());

    commands
        .spawn(SpriteBundle {
            sprite: title_handle,
            position: title_pos,
            stylemap: asset_server.load("demo/title.stylemap"),
            ..Default::default()
        })
        .with(Parent(*scene_root))
        .spawn(SpriteBundle {
            sprite: sprites.add(welcome_sprite),
            position: welcome_pos,
            stylemap: stylemaps.add(StyleMap::with_attribs(Attributes::from(
                &[Attribute::Bold, Attribute::Underlined][..],
            ))),
            ..Default::default()
        })
        .with(Parent(*scene_root))
        .spawn(SpriteBundle {
            sprite: sprites.add(explain_sprite),
            position: explain_pos,
            stylemap: color.clone(),
            ..Default::default()
        })
        .with(Parent(*scene_root))
        .spawn(SpriteBundle {
            sprite: sprites.add(press_sprite),
            position: press_pos,
            stylemap: stylemaps.add(StyleMap::with_attrib(Attribute::Italic)),
            ..Default::default()
        })
        .with(Parent(*scene_root));
}
