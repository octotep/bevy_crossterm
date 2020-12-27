use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use std::default::Default;

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = WindowSettings::default();
    settings.set_title("Window example");

    App::build()
        // Add our window settings
        .add_resource(settings)
        // Set some options in bevy to make our program a little less resource intensive - it's just a terminal game
        // no need to try and go nuts
        .add_resource(bevy::core::DefaultTaskPoolOptions::with_num_threads(1))
        // The Crossterm runner respects the schedulerunnersettings. No need to run as fast as humanly
        // possible - 20 fps should be more than enough for a scene that never changes
        .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_millis(50),
        ))
        // Add the DefaultPlugins before the CrosstermPlugin. The crossterm plugin needs bevy's asset server, and if it's
        // not available you'll trigger an assert
        .add_plugins(DefaultPlugins)
        .add_plugin(CrosstermPlugin)
        .add_startup_system(startup_system.system())
        .run();
}

fn startup_system(
    commands: &mut Commands,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    // Create our resources - two sprites and the default colors that we'll use for both
    let text = sprites.add(Sprite::new("This is an example which creates a crossterm window,\nsets a title, and displays some text."));
    let ctrlc = sprites.add(Sprite::new("Press Control-C to quit"));
    let color = stylemaps.add(StyleMap::default());

    // Spawn two sprites into the world
    commands
        .spawn(SpriteBundle {
            sprite: text,
            stylemap: color.clone(),
            ..Default::default()
        })
        .spawn(SpriteBundle {
            sprite: ctrlc,
            position: Position { x: 0, y: 3, z: 0 },
            stylemap: color,
            ..Default::default()
        });
}
