//! LFO panel: 4 modulation slots (enable/shape/target/rate/amount), each
//! routable to any registered `CommandId` of kind `Range`: driven by
//! `LfoEngine` (already ported+tested in `core::lfo`; this step is the
//! first to actually instantiate and drive it, see that module's own doc
//! comment). Port of `SidebarLfo.svelte`.
//!
//! Same "direct field mutation, no `CommandRegistry::dispatch`" convention
//! as every other panel for the panel's *own* controls (enable/shape/
//! target/rate/amount write straight into `Show::lfo_engine.slots[i]`,
//! same shape as `ui::strobe`'s rate/intensity/color); only the
//! modulation the engine *produces* goes through the registry, in
//! `app::about_to_wait`'s per-frame `LfoEngine::tick` loop. The target
//! dropdown is built from `CommandRegistry::all()` filtered to
//! `CommandKind::Range`, not a hardcoded list, so every Range command any
//! earlier step wired up (Color/Composite/Time/Qvar and beyond) shows up
//! here automatically.

use opendrop_core::commands::{CommandId, CommandKind, CommandRegistry};
use opendrop_core::lfo::LfoShape;
use opendrop_core::show::Show;

use crate::ui::widgets::{self, theme};

const SHAPES: [(LfoShape, &str); 4] =
    [(LfoShape::Sine, "Sine"), (LfoShape::Saw, "Saw"), (LfoShape::Square, "Square"), (LfoShape::Sh, "S&H")];

/// Discrete `rate` choices offered in place of a free-dragging slider,
/// labeled the way a musician would read them rather than as a raw
/// multiplier: `×n` (n cycles per beat, fast) on one side of the default
/// `×1`, `/n` (one cycle every n beats, slow/tempo-locked sweep) on the
/// other. `LfoSlot::rate` is "cycles per beat" either way (`×n` is `rate =
/// n`, `/n` is `rate = 1.0/n`); see that field's own doc comment for why a
/// below-1 rate needs `Clock::total_beats`/`LfoEngine::tick`'s
/// continuously-increasing phase to actually span multiple beats instead
/// of retracing the same truncated arc every single beat.
const LFO_RATE_OPTIONS: [(&str, f64); 15] = [
    ("×128", 128.0),
    ("×64", 64.0),
    ("×32", 32.0),
    ("×16", 16.0),
    ("×8", 8.0),
    ("×4", 4.0),
    ("×2", 2.0),
    ("×1", 1.0),
    ("/2", 1.0 / 2.0),
    ("/4", 1.0 / 4.0),
    ("/8", 1.0 / 8.0),
    ("/16", 1.0 / 16.0),
    ("/32", 1.0 / 32.0),
    ("/64", 1.0 / 64.0),
    ("/128", 1.0 / 128.0),
];

/// Registered `CommandId`s of kind `Range`, with their labels: the LFO
/// target dropdown's candidate list. A free function (not inlined into
/// `show`) so it's independently testable without an `egui::Ui`.
fn range_targets(registry: &CommandRegistry) -> Vec<(CommandId, &str)> {
    registry.all().into_iter().filter(|cmd| cmd.kind == CommandKind::Range).map(|cmd| (cmd.id, cmd.label)).collect()
}

pub fn show(ui: &mut egui::Ui, show: &mut Show, registry: &CommandRegistry) {
    ui.heading("LFO");

    let targets = range_targets(registry);

    for i in 0..show.lfo_engine.slots.len() {
        ui.push_id(i, |ui| {
            ui.separator();
            let slot = &mut show.lfo_engine.slots[i];

            ui.checkbox(&mut slot.enabled, format!("Slot {}", i + 1));

            ui.horizontal(|ui| {
                let t = theme(ui);
                for (shape, label) in SHAPES {
                    let color = if slot.shape == shape { t.palette.accent } else { t.palette.dim };
                    if widgets::pill(ui, label, color).interact(egui::Sense::click()).clicked() {
                        slot.shape = shape;
                    }
                }
            });

            widgets::micro_label(ui, "Target");
            let selected_text =
                slot.target.and_then(|id| targets.iter().find(|(tid, _)| *tid == id)).map_or("None", |(_, l)| *l);
            egui::ComboBox::from_id_salt("od_lfo_target").selected_text(selected_text).show_ui(ui, |ui| {
                if ui.selectable_label(slot.target.is_none(), "None").clicked() {
                    slot.target = None;
                }
                for (id, label) in &targets {
                    if ui.selectable_label(slot.target == Some(*id), *label).clicked() {
                        slot.target = Some(*id);
                    }
                }
            });

            widgets::micro_label(ui, "Rate");
            let selected_label = LFO_RATE_OPTIONS
                .iter()
                .find(|(_, rate)| (*rate - slot.rate).abs() < 1e-9)
                .map_or("Custom", |(label, _)| label);
            egui::ComboBox::from_id_salt("od_lfo_rate").selected_text(selected_label).show_ui(ui, |ui| {
                for (label, rate) in LFO_RATE_OPTIONS {
                    if ui.selectable_label(selected_label == label, label).clicked() {
                        slot.rate = rate;
                    }
                }
            });

            widgets::micro_label(ui, "Amount");
            ui.add(egui::Slider::new(&mut slot.amount, 0.0..=1.0).step_by(0.05));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::widgets::themed_test_ui;
    use opendrop_core::commands::create_default_registry;

    // `show` takes `Show` + `&CommandRegistry`, same testability tier as
    // `ui::strobe`/`ui::snapshot`/`ui::timeline`.

    #[test]
    fn show_does_not_panic() {
        themed_test_ui(|ui| {
            let mut state = Show::default();
            let registry = create_default_registry();
            show(ui, &mut state, &registry);
        });
    }

    #[test]
    fn show_does_not_panic_dense() {
        themed_test_ui(|ui| {
            widgets::dense(ui, |ui| {
                let mut state = Show::default();
                let registry = create_default_registry();
                show(ui, &mut state, &registry);
            });
        });
    }

    #[test]
    fn show_does_not_panic_with_every_slot_enabled_and_a_target_set() {
        let registry = create_default_registry();
        themed_test_ui(|ui| {
            let mut state = Show::default();
            for (i, slot) in state.lfo_engine.slots.iter_mut().enumerate() {
                slot.enabled = true;
                slot.shape = [LfoShape::Sine, LfoShape::Saw, LfoShape::Square, LfoShape::Sh][i % 4];
                slot.target = Some(CommandId::ColorHueA);
            }
            show(ui, &mut state, &registry);
        });
    }

    mod range_targets {
        use super::*;

        #[test]
        fn only_lists_range_kind_commands() {
            let registry = create_default_registry();
            let targets = super::range_targets(&registry);
            for (id, _) in &targets {
                assert_eq!(registry.get(*id).unwrap().kind, CommandKind::Range);
            }
        }

        #[test]
        fn excludes_trigger_kind_commands() {
            let registry = create_default_registry();
            let targets = super::range_targets(&registry);
            assert!(!targets.iter().any(|(id, _)| *id == CommandId::DeckSwitch)); // Trigger, not Range
        }

        #[test]
        fn includes_at_least_one_color_composite_time_and_qvar_target() {
            // Minimum bar, at the panel-building level: the same 4
            // families `core::show`'s `lfo_end_to_end_dispatch` tests
            // drive through the registry must actually be selectable here.
            let registry = create_default_registry();
            let targets = super::range_targets(&registry);
            let ids: Vec<CommandId> = targets.iter().map(|(id, _)| *id).collect();
            assert!(ids.contains(&CommandId::ColorHueA));
            assert!(ids.contains(&CommandId::CompositeBlend0));
            assert!(ids.contains(&CommandId::TimeSpeed0));
            assert!(ids.contains(&CommandId::Qvar1_0));
        }
    }
}
