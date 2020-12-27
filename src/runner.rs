use std::io::Write;

use crate::{CrosstermWindow, WindowSettings};

use bevy::app::{App, AppExit, Events};
use bevy::window::{WindowCreated, WindowId, WindowResized};
use crossterm::{ExecutableCommand, QueueableCommand};

pub fn crossterm_runner(mut app: App) {
    let settings = app
        .resources
        .get_or_insert_with(bevy::app::ScheduleRunnerSettings::default)
        .to_owned();
    let window_settings = app
        .resources
        .get_or_insert_with(WindowSettings::default)
        .clone();

    let mut term = std::io::stdout();

    term.queue(crossterm::terminal::EnterAlternateScreen)
        .unwrap();
    term.queue(crossterm::event::EnableMouseCapture).unwrap();

    crossterm::terminal::enable_raw_mode().expect("Could not enable crossterm raw mode");

    let mut window = CrosstermWindow::default();

    // Use settings in window
    {
        if let Some(title) = &window_settings.title() {
            window.title = Some(title.clone());
            term.queue(crossterm::terminal::SetTitle(&title))
                .expect("Could not set terminal title");
        }

        window.colors = window_settings.colors();
        term.queue(crossterm::style::SetColors(window.colors.to_crossterm()))
            .expect("Could not set window colors");
    }

    // Insert our window resources so that other parts of our app can use them
    app.resources.insert(window);

    term.queue(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))
    .expect("Could not clear screen");

    term.flush().unwrap();

    // Publish to the app that a terminal window has been created
    {
        let mut window_created_events = app.resources.get_mut::<Events<WindowCreated>>().unwrap();
        window_created_events.send(WindowCreated {
            id: WindowId::primary(),
        });
    }

    match settings.run_mode {
        bevy::app::RunMode::Once => {
            app.update();
        }
        bevy::app::RunMode::Loop { wait } => {
            // Main loop
            let tick = move |app: &mut App,
                             wait: Option<std::time::Duration>|
                  -> Result<Option<std::time::Duration>, AppExit> {
                let start_time = std::time::Instant::now();

                // Check if any events are immediately available and if so, read them and republish
                while let Ok(available) = crossterm::event::poll(std::time::Duration::from_secs(0))
                {
                    if available {
                        match crossterm::event::read().unwrap() {
                            // Republish keyboard events in bevy
                            crossterm::event::Event::Key(key_event) => {
                                // If the key event is for C-c, submit a AppExit event so the application
                                // can be killed
                                use crossterm::event::{KeyCode, KeyModifiers};
                                if key_event.code == KeyCode::Char('c')
                                    && key_event.modifiers.contains(KeyModifiers::CONTROL)
                                {
                                    let mut app_exit_events =
                                        app.resources.get_mut::<Events<AppExit>>().unwrap();
                                    app_exit_events.send(AppExit);
                                }

                                let mut bevy_key_events = app
                                    .resources
                                    .get_mut::<Events<crossterm::event::KeyEvent>>()
                                    .unwrap();
                                bevy_key_events.send(key_event);
                            }

                            // Republish mouse events in bevy
                            crossterm::event::Event::Mouse(mouse_event) => {
                                let mut bevy_mouse_events = app
                                    .resources
                                    .get_mut::<Events<crossterm::event::MouseEvent>>()
                                    .unwrap();
                                bevy_mouse_events.send(mouse_event);
                            }

                            // Send a bevy window resized event if the terminal is resized, and also change the persisted window state
                            crossterm::event::Event::Resize(width, height) => {
                                // Update the window resource and publish an event for the window being resized
                                let mut window_resized_events =
                                    app.resources.get_mut::<Events<WindowResized>>().unwrap();
                                window_resized_events.send(WindowResized {
                                    id: WindowId::primary(),
                                    width: width as f32,
                                    height: height as f32,
                                });

                                let mut window =
                                    app.resources.get_mut::<CrosstermWindow>().unwrap();
                                window.height = height;
                                window.width = width;
                            }
                        }
                    } else {
                        break;
                    }
                }

                // Yield execution to the rest of bevy and it's scheduler
                app.update();

                // After all the other systems have updated, check if there are any AppExit events and
                // handle them
                {
                    let app_exit_events = app.resources.get::<Events<AppExit>>().unwrap();
                    let mut app_exit_reader = app_exit_events.get_reader();
                    if app_exit_reader.iter(&app_exit_events).next().is_some() {
                        // We're breaking out, the app requested an exit
                        return Err(AppExit);
                    };
                }

                let end_time = std::time::Instant::now();

                // Calculate how much time that took vs how much time we wanted to wait
                if let Some(wait) = wait {
                    let exe_time = end_time - start_time;
                    if exe_time < wait {
                        return Ok(Some(wait - exe_time));
                    }
                }

                Ok(None)
            };

            // Run the main loop, and delay if we need to
            while let Ok(delay) = tick(&mut app, wait) {
                if let Some(delay) = delay {
                    std::thread::sleep(delay);
                }
            }

            // Cleanup and teardown
            term.execute(crossterm::event::DisableMouseCapture)
                .expect("Could not disable mouse capture");
            crossterm::terminal::disable_raw_mode().expect("Could not disable raw mode");
            term.execute(crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            term.execute(crossterm::cursor::Show).unwrap();
        }
    }
}
