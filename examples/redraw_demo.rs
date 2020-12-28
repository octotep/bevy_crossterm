use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use std::default::Default;

// This is probably the busiest example. This demonstrates that bevy_crossterm's incrememntal drawing
// system will properly redraw sprites when required. To simulate movement - bevy_crossterm blanks out the
// previous position/size of any sprites that changed before they're drawn again. This leave empty holes if
// a sprite changed while over top of another sprite. This is illustrated by the "@" going on top of the big box.
// bevy_crossterm runs a collision detection routine and sees that the box is going to be partially erased when
// the "@" is cleared, so it adds it to the draw candidate list even though it never "changed".

// Furthermore, this example also illustrates that the collision detection happens recusively - if it weren't for that,
// the "#" would be overwritten when the box is redrawn.

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Redraw example");

    App::build()
        .add_resource(settings)
        .add_resource(bevy::core::DefaultTaskPoolOptions::with_num_threads(1))
        // 60Hz update is probably a bit gratuitous for this but eh
        .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_millis(16),
        ))
        .add_resource(Timer::new(std::time::Duration::from_millis(250), true))
        .add_plugins(DefaultPlugins)
        .add_plugin(CrosstermPlugin)
        .add_startup_system(startup_system.system())
        .add_system(update.system())
        .run();
}

// 5x5 box of spaces
static BIG_BOX: &str = "       \n       \n       ";

struct Tag;

fn startup_system(
    commands: &mut Commands,
    window: Res<CrosstermWindow>,
    mut cursor: ResMut<Cursor>,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    cursor.hidden = true;

    // Create our resources
    let big_box = sprites.add(Sprite::new(BIG_BOX));
    let small_box = sprites.add(Sprite::new("@"));
    let plain = stylemaps.add(StyleMap::default());
    let white_bg = stylemaps.add(StyleMap::with_bg(Color::White));

    // Spawn two sprites into the world
    commands
        .spawn(SpriteBundle {
            sprite: big_box,
            position: Position {
                x: window.x_center() as i32 - 3,
                y: window.y_center() as i32 - 1,
                z: 0,
            },
            stylemap: white_bg.clone(),
            ..Default::default()
        })
        // Moving entity that ensures the box will get redrawn each step the entity passes over it
        .spawn(SpriteBundle {
            sprite: small_box,
            position: Position {
                x: window.width() as i32 / 3,
                y: window.y_center() as i32,
                z: 1,
            },
            stylemap: plain.clone(),
            ..Default::default()
        })
        .with(Tag); // Tagged with a unit struct so we can find it later to update it
                    // Static entity that ensures we redraw all entities that need to
    commands.spawn(SpriteBundle {
        sprite: sprites.add(Sprite::new("#")),
        position: Position {
            x: window.x_center() as i32,
            y: window.y_center() as i32 - 1,
            z: 1,
        },
        stylemap: plain.clone(),
        ..Default::default()
    });
}

fn update(
    time: Res<Time>,
    window: Res<CrosstermWindow>,
    mut timer: ResMut<Timer>,
    mut query: Query<(&Tag, &mut Position)>,
    mut app_exit: ResMut<Events<AppExit>>,
) {
    timer.tick(time.delta_seconds());

    if timer.just_finished() {
        let (_, mut pos) = query.iter_mut().next().unwrap();
        pos.x += 1;

        if pos.x > (window.width() as i32 / 3 * 2) {
            app_exit.send(AppExit);
        }
    }
}
