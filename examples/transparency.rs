use std::time;

use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;

use bevy_crossterm::prelude::*;

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Transparency example");

    App::build()
        .insert_resource(settings)
        .insert_resource(DefaultTaskPoolOptions::with_num_threads(1))
        .insert_resource(ScheduleRunnerSettings::run_loop(time::Duration::from_millis(6)))
        .insert_resource(Timer::new(time::Duration::from_millis(10), true))
        .add_plugins(DefaultCrosstermPlugins)
        .add_startup_system(startup_system.system())
        .run();
}

// 5x5 box of spaces
static BIG_BOX: &str = "         \n         \n         \n         \n         ";
static SMALL_BOX: &str = r##"@@@@@
@ @ @
@   @
@ @ @
@@@@@"##;

fn startup_system(mut commands: Commands, window: Res<CrosstermWindow>, mut cursor: ResMut<Cursor>, mut sprites: ResMut<Assets<Sprite>>, mut stylemaps: ResMut<Assets<StyleMap>>) {
    cursor.hidden = true;

    // Create our resources
    let plain = stylemaps.add(StyleMap::default());
    let white_bg = stylemaps.add(StyleMap::with_bg(Color::White));

    // Spawn two sprites into the world
    commands.spawn_bundle(SpriteBundle {
        sprite: sprites.add(Sprite::new(BIG_BOX)),
        position: Position {
            x: window.x_center() as i32 - 3,
            y: window.y_center() as i32 - 1,
            z: 0,
        },
        stylemap: white_bg,
        ..Default::default()
    });
    commands
        // Moving entity that ensures the box will get redrawn each step the entity passes over it
        .spawn_bundle(SpriteBundle {
            sprite: sprites.add(Sprite::new(SMALL_BOX)),
            position: Position {
                x: window.x_center() as i32 - 1,
                y: window.y_center() as i32 - 1,
                z: 1,
            },
            stylemap: plain,
            visible: Visible::transparent(),
        });
}
