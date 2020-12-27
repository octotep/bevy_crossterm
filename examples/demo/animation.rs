use bevy::prelude::*;
use bevy_crossterm::prelude::*;

pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

pub fn setup(
    commands: &mut Commands,
    scene_root: Res<Entity>,
    window: Res<CrosstermWindow>,
    asset_server: Res<AssetServer>,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    let default_style = stylemaps.add(StyleMap::default());
    let white = stylemaps.add(StyleMap::with_bg(Color::White));

    let text_sprite = Sprite::new("If you modify the position, the sprite will be rerendered at the new position.\nbevy_crossterm draws incrementally, so any sprites that change are erased then drawn at their new position.\nAll other sprites affected by the erasing step are redrawn too.\nThis cuts down on the amount of things to redraw in a single frame.");
    let text_pos = Position::with_x(
        window.x_center() as i32 - text_sprite.x_center() as i32,
    );

    let hor_divider = Sprite::new("─".repeat(window.width() as usize));
    let divider_pos = Position::with_y(text_sprite.height() as i32);

    let test_box = Sprite::new("           \n           \n           \n           \n           ");
    let test_pos = Position::with_xy(
        window.x_center() as i32 - test_box.x_center() as i32,
        window.y_center() as i32 - test_box.y_center() as i32,
    );

    commands
        .spawn(SpriteBundle {
            sprite: sprites.add(text_sprite),
            position: text_pos,
            stylemap: default_style.clone(),
            ..Default::default()
        })
        .with(Parent(*scene_root))

        .spawn(SpriteBundle {
            sprite: sprites.add(hor_divider),
            position: divider_pos,
            stylemap: default_style.clone(),
            ..Default::default()
        })
        .with(Parent(*scene_root))

        .spawn(SpriteBundle {
            sprite: sprites.add(test_box),
            stylemap: white.clone(),
            position: test_pos,
            ..Default::default()
        })
        .with(Parent(*scene_root))

        .spawn(SpriteBundle {
            sprite: asset_server.get_handle("demo/bounce.txt"),
            stylemap: asset_server.get_handle("demo/bounce.stylemap"),
            position: Position::new(
                window.x_center() as i32,
                window.y_center() as i32,
                1
            ),
            ..Default::default()
        })
        .with(Parent(*scene_root))
        .with(Velocity {
            x: 1,
            y: 1,
        });

    commands.insert_resource(Timer::new(std::time::Duration::from_millis(120), true));
}

pub fn update(
    keys: Res<Events<KeyEvent>>,
    mut state: ResMut<State<crate::GameState>>,
    mut app_exit: ResMut<Events<bevy::app::AppExit>>,
    mut timer: ResMut<Timer>,
    window: Res<CrosstermWindow>,
    time: Res<Time>,
    sprites: Res<Assets<Sprite>>,
    mut box_sprite: Query<(&mut Position, &mut Velocity, &Handle<Sprite>)>,
) {
    timer.tick(time.delta_seconds());

    if crate::detect_keypress(keys) {
        if let Some(new_state) = state.next_state() {
            state.set_next(new_state).unwrap();
        } else {
            app_exit.send(bevy::app::AppExit);
        }
        return;
    }

    if timer.just_finished() {
        let (mut pos, mut vel, sprite) = box_sprite.iter_mut().next().unwrap();
        let sprite = sprites.get(sprite).unwrap();

        pos.x += vel.x;
        pos.y += vel.y;

        if pos.x < 0 {
            pos.x = 0;
            vel.x = 1;
        }

        if pos.x > (window.width() as i32 - sprite.width() as i32) {
            pos.x = window.width() as i32 - sprite.width() as i32;
            vel.x = -1;
        }

        // Leave room for the header
        if pos.y < 5 {
            pos.y = 5;
            vel.y = 1;
        }

        if pos.y > (window.height() as i32 - sprite.height() as i32) {
            pos.y = window.height() as i32 - sprite.height() as i32;
            vel.y = -1;
        }
    }
}