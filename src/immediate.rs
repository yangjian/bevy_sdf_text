use bevy::ecs::change_detection::Tick;
use bevy::ecs::system::{SystemChangeTick, SystemParam};
use bevy::prelude::*;

use crate::{SdfText, SdfTextAnchor, SdfTextBackground, SdfTextStyle};

#[derive(Clone, Debug, Default)]
pub struct SdfTextPainterConfig {
    pub style: SdfTextStyle,
    pub anchor: SdfTextAnchor,
    pub transform: Transform,
    pub background: SdfTextBackground,
}

#[derive(Default)]
struct TextPainterState {
    entities: Vec<Entity>,
    cursor: usize,
    run_tick: Option<u32>,
    config: SdfTextPainterConfig,
}

#[derive(Component)]
pub(crate) struct TextPainterDrawn(Tick);

/// Immediate-mode text drawing helper for Bevy systems.
///
/// `TextPainter` reuses one hidden entity per draw call position in the containing system.
/// Call [`draw`](TextPainter::draw) every frame for each text item that should be visible.
#[derive(SystemParam)]
pub struct SdfTextPainter<'w, 's> {
    commands: Commands<'w, 's>,
    change_tick: SystemChangeTick,
    state: Local<'s, TextPainterState>,
}

impl SdfTextPainter<'_, '_> {
    pub fn config(&self) -> &SdfTextPainterConfig {
        &self.state.config
    }

    pub fn config_mut(&mut self) -> &mut SdfTextPainterConfig {
        &mut self.state.config
    }

    pub fn draw(&mut self, text: &str) {
        let config = self.state.config.clone();
        self.draw_with_config(text, &config);
    }

    fn draw_with_config(&mut self, text: &str, config: &SdfTextPainterConfig) {
        let run_tick = self.change_tick.this_run().get();
        if self.state.run_tick != Some(run_tick) {
            self.state.cursor = 0;
            self.state.run_tick = Some(run_tick);
        }

        let entity = match self.state.entities.get(self.state.cursor) {
            Some(entity) => *entity,
            None => {
                let entity = self.commands.spawn_empty().id();
                self.state.entities.push(entity);
                entity
            }
        };
        self.state.cursor += 1;

        self.commands.entity(entity).insert((
            Name::new("SdfTextPainterText"),
            SdfText::from_section(text, config.style.clone()),
            config.anchor,
            config.transform,
            config.background.clone(),
            Visibility::Inherited,
            TextPainterDrawn(self.change_tick.this_run()),
        ));
    }
}

pub(crate) fn hide_stale_text_painter_entities(
    change_tick: SystemChangeTick,
    mut query: Query<(&TextPainterDrawn, &mut Visibility)>,
) {
    for (drawn, mut visibility) in &mut query {
        if !drawn
            .0
            .is_newer_than(change_tick.last_run(), change_tick.this_run())
        {
            *visibility = Visibility::Hidden;
        }
    }
}
