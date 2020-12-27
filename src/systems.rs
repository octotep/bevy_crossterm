use std::{convert::TryInto, io::Write};

use crate::components::{self, Style};
use crate::components::{
    Colors, EntityDepth, Position, PreviousEntityDetails, PreviousPosition, PreviousSize,
    PreviousWindowColors, Sprite, StyleMap, Visible,
};
use crate::{CrosstermWindow, Cursor};

use bevy::utils::HashSet;

use bevy::prelude::*;
use bevy::window::WindowResized;
use components::EntitiesToRedraw;
use crossterm::{ExecutableCommand, QueueableCommand};

use broccoli::prelude::*;

pub(crate) fn add_previous_position(
    mut entities_without_assets: Local<HashSet<Entity>>,
    mut previous_details: ResMut<PreviousEntityDetails>,
    frames: Res<Assets<Sprite>>,
    entities: Query<(Entity, &Position, &Handle<Sprite>), (Added<Position>, Added<Handle<Sprite>>)>,
    all: Query<(&Position, &Handle<Sprite>)>,
) {
    for (entity, pos, sprite) in entities.iter() {
        if let Some(sprite) = frames.get(&*sprite) {
            let prev_pos = components::PreviousPosition {
                x: pos.x,
                y: pos.y,
                z: pos.z,
            };
            let prev_size = components::PreviousSize {
                width: sprite.width() as u16,
                height: sprite.graphemes().len() as u16,
            };
            previous_details.0.insert(entity, (prev_pos, prev_size));
        } else {
            // The asset hasn't loaded yet, so let's make a record of it for later
            entities_without_assets.insert(entity);
        }
    }

    // Bevy loads assets asynchronously so if we couldn't find an asset, we add it to this list and
    // try again until it loads
    let mut entities_to_remove = Vec::new();
    for entity in entities_without_assets.iter() {
        let data = all.get(*entity);
        if data.is_err() {
            continue;
        }
        let (pos, sprite) = data.unwrap();

        if let Some(sprite) = frames.get(&*sprite) {
            let prev_pos = PreviousPosition {
                x: pos.x,
                y: pos.y,
                z: pos.z,
            };
            let prev_size = PreviousSize {
                width: sprite.width() as u16,
                height: sprite.graphemes().len() as u16,
            };
            previous_details.0.insert(*entity, (prev_pos, prev_size));

            // We need to remove this entity now, but can't since it's container is borrowed.
            {
                // Needs it's own scope or the borrow checker can't tell when thee mutable borrow ends
                entities_to_remove.push(*entity);
            }
        }
    }

    for entity in entities_to_remove.iter() {
        entities_without_assets.remove(entity);
    }
}

pub(crate) fn update_previous_position(
    mut previous_details: ResMut<PreviousEntityDetails>,
    frames: Res<Assets<Sprite>>,
    mut positions: Query<(Entity, &Position, &Handle<Sprite>, &Visible)>,
) {
    for (entity, new_pos, sprite, _) in positions.iter_mut() {
        if let Some(sprite) = frames.get(sprite) {
            let prev_pos = PreviousPosition {
                x: new_pos.x,
                y: new_pos.y,
                z: new_pos.z,
            };
            let prev_size = PreviousSize {
                width: sprite.width() as u16,
                height: sprite.graphemes().len() as u16,
            };

            if let Some(value) = previous_details.0.get_mut(&entity) {
                *value = (prev_pos, prev_size);
            }
        }
    }
}

pub(crate) fn calculate_entities_to_redraw(
    mut prev_colors: ResMut<PreviousWindowColors>,
    mut entities: ResMut<EntitiesToRedraw>,
    previous_details: Res<PreviousEntityDetails>,
    window: Res<CrosstermWindow>,
    resize_events: Res<Events<WindowResized>>,
    sprites: Res<Assets<Sprite>>,
    sprite_asset_events: Res<Events<AssetEvent<Sprite>>>,
    stylemap_asset_events: Res<Events<AssetEvent<StyleMap>>>,
    all: Query<(
        Entity,
        &Handle<StyleMap>,
        &Handle<Sprite>,
        &Position,
        &Visible,
    )>,
    changed: Query<
        Entity,
        Or<(
            Mutated<Position>,
            Mutated<Handle<StyleMap>>,
            Mutated<Visible>,
            Mutated<Handle<Sprite>>,
        )>,
    >,
    added: Query<
        Entity,
        Or<(
            Added<Position>,
            Added<Handle<StyleMap>>,
            Added<Visible>,
            Added<Handle<Sprite>>,
        )>,
    >,
) {
    entities.full_redraw = false;
    entities.to_draw.clear();
    entities.to_clear.clear();

    let mut draw_set = HashSet::default();

    // If a resize happened the whole screen is invalidated
    if resize_events.get_reader().latest(&resize_events).is_some() || window.colors != prev_colors.0
    {
        // We need a full redraw, so flag a full update and bail early
        // No need to do fancy update calculations
        entities.full_redraw = true;
        prev_colors.0 = window.colors;
        // Mark all entities as needed to redraw
        for (entity, _, _, pos, _) in all.iter() {
            entities.to_draw.push(EntityDepth { entity, z: pos.z });
        }
        entities.to_draw.sort_by_key(|item| item.z);
        return;
    }

    // Now check to see which entities actually changed since it's not a full update

    // Collect all the changed assets
    let mut created_sprite_assets = bevy::utils::HashSet::default();
    let mut changed_sprite_assets = bevy::utils::HashSet::default();
    let mut created_stylemap_assets = bevy::utils::HashSet::default();
    let mut changed_stylemap_assets = bevy::utils::HashSet::default();
    for evt in sprite_asset_events.get_reader().iter(&sprite_asset_events) {
        match &*evt {
            AssetEvent::Created { handle } => {
                created_sprite_assets.insert(handle.clone());
            }
            AssetEvent::Modified { handle } => {
                changed_sprite_assets.insert(handle.clone());
            }
            _ => {}
        }
    }
    for evt in stylemap_asset_events
        .get_reader()
        .iter(&stylemap_asset_events)
    {
        match &*evt {
            AssetEvent::Created { handle } => {
                created_stylemap_assets.insert(handle.clone());
            }
            AssetEvent::Modified { handle } => {
                changed_stylemap_assets.insert(handle.clone());
            }
            _ => {}
        }
    }

    // Collect all the entities that changed this update, either because their asset did,
    // or their components did
    for (entity, style_hnd, sprite_hnd, _, _) in all.iter() {
        if changed_sprite_assets.contains(sprite_hnd) || changed_stylemap_assets.contains(style_hnd)
        {
            entities.to_clear.insert(entity);
            draw_set.insert(entity);
        }

        if created_sprite_assets.contains(sprite_hnd) || created_stylemap_assets.contains(style_hnd)
        {
            draw_set.insert(entity);
        }
    }

    for entity in changed.iter() {
        entities.to_clear.insert(entity);
        draw_set.insert(entity);
    }

    for entity in added.iter() {
        draw_set.insert(entity);
    }

    // Find all entities that either became invisible, or changed their size or moved. (cleared is good enough for now)
    // Figure out what their previous bounding box is and query all current positions to see what sprites are under it
    // Add the collided entities to draw_set
    let mut new_ents = Vec::new();
    let mut bboxes = Vec::new();
    for (entity, _, sprite, pos, _) in all.iter() {
        let sprite_data = sprites.get(sprite);
        if sprite_data.is_none() {
            continue;
        }
        let sprite = sprite_data.unwrap();
        let bb = broccoli::bbox(
            broccoli::rect(
                pos.x,
                pos.x + sprite.width() as i32,
                pos.y,
                pos.y + sprite.data().len() as i32,
            ),
            entity,
        );
        bboxes.push(bb);
    }

    let broccoli = broccoli::new(&mut bboxes);
    for ent in changed.iter() {
        let prev_data = previous_details.0.get(&ent);
        if prev_data.is_none() {
            continue;
        }
        let (prev_pos, prev_size) = prev_data.unwrap();
        let blank_bb = broccoli::rect(
            prev_pos.x,
            prev_pos.x + prev_size.width as i32,
            prev_pos.y,
            prev_pos.y + prev_size.height as i32,
        );
        // dbg!("checking for collision", ent, prev_pos);
        broccoli.for_all_intersect_rect(&blank_bb, |bb| {
            if ent == bb.inner {
                return;
            }
            // dbg!("Found Entity: ", bb.inner);
            if !draw_set.contains(&bb.inner) {
                draw_set.insert(bb.inner);
                new_ents.push(bb.inner);
            }
        });
    }

    let mut cur_index = 0;
    while cur_index < new_ents.len() {
        let ent = new_ents[cur_index];
        cur_index += 1;

        let prev_data = previous_details.0.get(&ent);
        if prev_data.is_none() {
            continue;
        }
        let (prev_pos, prev_size) = prev_data.unwrap();
        let blank_bb = broccoli::rect(
            prev_pos.x,
            prev_pos.x + prev_size.width as i32,
            prev_pos.y,
            prev_pos.y + prev_size.height as i32,
        );
        // dbg!("checking for collision", ent, prev_pos);
        broccoli.for_all_intersect_rect(&blank_bb, |bb| {
            if ent == bb.inner {
                return;
            }
            // dbg!("Found Entity: ", bb.inner);
            if !draw_set.contains(&bb.inner) {
                draw_set.insert(bb.inner);
                new_ents.push(bb.inner);
            }
        });
    }

    let removed = all.removed::<Handle<Sprite>>();

    for entity in removed {
        entities.to_clear.insert(*entity);
    }

    for ent_to_draw in draw_set.iter() {
        let (entity, _, _, pos, _) = all.get(*ent_to_draw).unwrap();
        entities
            .to_draw
            .push(components::EntityDepth { entity, z: pos.z });
    }
    entities.to_draw.sort_by_key(|item| item.z);
}

/// Helper function for draw_entity which determines whether the style on the terminal should be
/// changed
fn change_style_if_needed(
    term: &mut std::io::StdoutLock,
    previous_style: &mut Style,
    current_style: &Style,
) -> Result<(), Box<dyn std::error::Error>> {
    if current_style.attributes != previous_style.attributes {
        term.queue(crossterm::style::SetAttributes(current_style.attributes))?;
        previous_style.attributes = current_style.attributes;
    }
    if current_style.colors != previous_style.colors {
        term.queue(crossterm::style::SetColors(
            current_style.colors.to_crossterm(),
        ))?;
        previous_style.colors = current_style.colors;
    }
    Ok(())
}

fn draw_entity(
    entity: Entity,
    term: &mut std::io::StdoutLock,
    window: &CrosstermWindow,
    sprites: &Res<Assets<Sprite>>,
    stylemaps: &Res<Assets<StyleMap>>,
    all: &Query<(
        Entity,
        &Position,
        &Handle<StyleMap>,
        &components::Visible,
        &Handle<Sprite>,
    )>,
) -> Result<(), Box<dyn std::error::Error>> {
    let entity_data = all.get(entity);
    if entity_data.is_err() {
        return Ok(());
    }
    let (_, pos, style, draw, sprite) = entity_data.unwrap();

    // If the entity isn't visible, skip it
    if !draw.is_visible {
        return Ok(());
    }

    let sprite = sprites.get(&*sprite);
    if sprite.is_none() {
        // The sprite asset hasn't loaded yet, this isn't a problem
        return Ok(());
    }
    let sprite = sprite.unwrap();

    // If the entity's not on the screen, skip it
    if pos.y >= window.height.into()
        || pos.y + sprite.height() as i32 <= 0
        || pos.x >= window.width.into()
        || pos.x + sprite.width() as i32 <= 0
    {
        return Ok(());
    }

    let stylemap = stylemaps.get(&*style);
    if stylemap.is_none() {
        // The stylemap asset hasn't loaded yet, this isn't a problem
        return Ok(());
    }
    let stylemap = stylemap.unwrap();
    let sprite_colors = stylemap.style.colors.with_default(window.colors);

    term
        .queue(crossterm::style::SetAttribute(crossterm::style::Attribute::Reset))?
        .queue(crossterm::style::SetAttributes(stylemap.style.attributes))?
        .queue(crossterm::style::SetColors(sprite_colors.to_crossterm()))?;

    let mut previous_style = stylemap.style;

    for (line_num, line) in sprite.graphemes().iter().enumerate() {
        let line_offset: i32 = line_num.try_into()?;

        // Check to see if this line is on the screen, if not skip it
        if pos.y + line_offset < 0 {
            continue;
        }

        // If this line is off the bottom of the screen, break out since no lines can ever
        // be on the screen ever again
        if pos.y + line_offset >= window.height.into() {
            break;
        }

        // Calculate the beginning and end of string sprte, to not render things off screen
        let start: i32 = std::cmp::max(0, pos.x).try_into()?;
        let end: i32 = std::cmp::min(window.width as i32, pos.x + line.len() as i32);

        let start_idx: usize = (start - pos.x).try_into()?;
        let end_idx: usize = (end - pos.x).try_into()?;

        term.queue(crossterm::cursor::MoveTo(
            start.try_into()?,
            (pos.y + line_offset).try_into()?,
        ))?;

        let graphemes = &line[start_idx..end_idx];
        if graphemes.len() != 0 {
            // Go through each grapheme one by one to make sure we have the correct style and color
            // (Cross reference with the stylemap, otherwise default to )
            for (i, grapheme) in graphemes.iter().enumerate() {
                let idx = start_idx + i;

                // We need to skip the grapheme if it's a space and supposed to be transparent
                if draw.is_transparent {
                    if let Some(_) = stylemap.style_at(idx, line_num) {
                        // Just fallthrough
                    } else {
                        // If the grapheme is a transparent space with no style, skip it
                        if sprite.grapheme(grapheme) == " " {
                            term.queue(crossterm::cursor::MoveRight(1))?;
                            continue;
                        }
                    }
                }

                // Get the style we need to render this grapheme with
                let grapheme_style = stylemap.style_for(idx, line_num);
                change_style_if_needed(term, &mut previous_style, &grapheme_style)?;

                term.queue(crossterm::style::Print(&sprite.grapheme(grapheme)))?;
            }
        }

        // Lines don't have to go to the end of the sprite. Pad them out so the sprite is rectangular
        if end < window.width as i32 && line.len() < sprite.width() {
            let unaccounted = sprite.width() - line.len();
            let blank_length = std::cmp::min(unaccounted, (window.width as i32 - end) as usize);
            let blank_str = str::repeat(" ", blank_length);
            for (i, space) in blank_str.chars().enumerate() {
                let idx = end_idx + i;

                // We need to skip the grapheme if it's a space and supposed to be transparent
                if draw.is_transparent {
                    if let Some(_) = stylemap.style_at(idx, line_num) {
                        // Just fallthrough
                    } else {
                        // The grapheme is a transparent space with no style, so skip it
                        term.queue(crossterm::cursor::MoveRight(1))?;
                        continue;
                    }
                }

                // Get the style we need to render this space with
                let grapheme_style = stylemap.style_for(idx, line_num);
                change_style_if_needed(term, &mut previous_style, &grapheme_style)?;

                term.queue(crossterm::style::Print(space))?;
            }
        }
    }

    Ok(())
}

fn clear_entity(
    entity: Entity,
    term: &mut std::io::StdoutLock,
    window: &CrosstermWindow,
    previous_details: &PreviousEntityDetails,
) -> Result<(), Box<dyn std::error::Error>> {
    let prev_details = previous_details.0.get(&entity);
    if prev_details.is_none() {
        // We didn't have a chance to create previous details for this entity yet
        // Since it's sprite is most likely still loading
        return Ok(());
    }
    let (prev_pos, prev_size) = prev_details.unwrap();

    for height in 0..prev_size.height {
        let y = prev_pos.y + height as i32;

        if y < 0 {
            continue;
        }

        if prev_pos.y >= window.height.into()
            || prev_pos.y + prev_size.height as i32 <= 0
            || prev_pos.x >= window.width.into()
            || prev_pos.x + prev_size.width as i32 <= 0
        {
            break;
        }

        let x_start: i32 = std::cmp::max(0, prev_pos.x).try_into()?;
        let x_end: i32 = std::cmp::min(window.width as i32, prev_pos.x + prev_size.width as i32);

        let actual_width = x_end - x_start;
        let blank_string = " ".repeat(actual_width as usize);

        term.queue(crossterm::style::SetAttribute(
            crossterm::style::Attribute::Reset,
        ))?
        .queue(crossterm::style::SetColors(
            Colors::term_colors().to_crossterm(),
        ))?
        .queue(crossterm::cursor::MoveTo(
            x_start.try_into()?,
            y.try_into()?,
        ))?
        .queue(crossterm::style::Print(blank_string))?;
    }

    Ok(())
}

// TODO: Redraw entities if they were under another entity that became invisible or was deleted, or moves
pub(crate) fn crossterm_render(
    changed_entities: Res<EntitiesToRedraw>,
    window: Res<CrosstermWindow>,
    cursor: Res<Cursor>,
    previous_details: Res<PreviousEntityDetails>,
    sprites: Res<Assets<Sprite>>,
    stylemaps: Res<Assets<StyleMap>>,
    all: Query<(
        Entity,
        &Position,
        &Handle<StyleMap>,
        &Visible,
        &Handle<Sprite>,
    )>,
) {
    let stdout = std::io::stdout();
    let mut term = stdout.lock();

    // If we're gonna be drawing stuff, hide the cursor so it doesn't jump all over the place
    if changed_entities.to_draw.len() > 0 {
        term.execute(crossterm::cursor::Hide).unwrap();
    }

    // If a resize happened, clear the screen and go from there
    if changed_entities.full_redraw {
        term
            .execute(crossterm::style::SetAttribute(crossterm::style::Attribute::Reset))
            .unwrap()
            .execute(crossterm::terminal::Clear(
                crossterm::terminal::ClearType::All,
            ))
            .unwrap();
    }

    // Blank out all the previous locations of sprites that changed either their position or their size
    for entity in changed_entities.to_clear.iter() {
        clear_entity(*entity, &mut term, &window, &previous_details).unwrap();
    }

    // Redraw all the changed sprites, either because they moved, or because they changed their shape
    // for entity in changed_entities.to_clear.iter() {
    //     draw_entity(*entity, &mut term, &window, &sprites, &stylemaps, &all).unwrap();
    // }
    for entity in changed_entities.to_draw.iter() {
        draw_entity(
            entity.entity,
            &mut term,
            &window,
            &sprites,
            &stylemaps,
            &all,
        )
        .unwrap();
    }

    // Draw the cursor at the right position, if needed
    if !cursor.hidden {
        if cursor.x >= 0
            && cursor.x < window.width as i32
            && cursor.y >= 0
            && cursor.y < window.height as i32
        {
            term.queue(crossterm::cursor::MoveTo(cursor.x as u16, cursor.y as u16))
                .unwrap();
            term.queue(crossterm::cursor::Show).unwrap();
        }
    } else {
        term.queue(crossterm::cursor::Hide).unwrap();
    }

    term.flush().unwrap();
}
