use std::time;

use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;

use bevy_crossterm::prelude::*;

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Window example");

    App::build()
        // Add our window settings
        .insert_resource(settings)
        // Set some options in bevy to make our program a little less resource intensive - it's just a terminal game
        // no need to try and go nuts
        .insert_resource(DefaultTaskPoolOptions::with_num_threads(1))
        // The Crossterm runner respects the schedulerunnersettings. No need to run as fast as humanly
        // possible - 20 fps should be more than enough for a scene that never changes
        .insert_resource(ScheduleRunnerSettings::run_loop(time::Duration::from_millis(50)))
        .add_plugins(DefaultCrosstermPlugins)
        .add_startup_system(startup_system.system())
        .run();
}

fn startup_system(mut commands: Commands, mut sprites: ResMut<Assets<Sprite>>, mut stylemaps: ResMut<Assets<StyleMap>>) {
    // Create our resources - two sprites and the default colors that we'll use for both
    let text = sprites.add(Sprite::new("This is an example which creates a crossterm window,\nsets a title, and displays some text."));
    let ctrlc = sprites.add(Sprite::new("Press Control-C to quit"));
    let color = stylemaps.add(StyleMap::default());

    // Spawn two sprites into the world
    commands.spawn_bundle(SpriteBundle {
        sprite: text,
        stylemap: color.clone(),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        sprite: ctrlc,
        position: Position { x: 0, y: 3, z: 0 },
        stylemap: color,
        ..Default::default()
    });
}
