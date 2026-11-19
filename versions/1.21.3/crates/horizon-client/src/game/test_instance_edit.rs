//! `TestInstanceBlockEditScreen`: the test's id, the size of the area, its
//! rotation and entity handling, and the reset, save and run actions.
use super::{
    Game,
    block_editor::{Editor, Item, Outcome, Pos, cycle, footer_action, on_off},
};
use horizon_client::{
    containers::BlockEntity,
    test_instance_block::TestInstanceBlock,
    text::{self, Text},
    ui::EditBox,
};
use horizon_protocol::packets::play::Serverbound as S;
use horizon_world::worldgen::structure::Rotation;

const ID: usize = 0;
const SIZE: usize = 1;

const RESET: u8 = 0;
const SAVE: u8 = 1;
const RUN: u8 = 2;
const DONE: u8 = 3;
const CANCEL: u8 = 4;
const ROTATION: u8 = 5;
const ENTITIES: u8 = 6;

/// The screen's width, and the gap between its buttons.
const WIDTH: f32 = 316.0;
const PADDING: f32 = 8.0;

pub(super) struct TestInstanceEditor {
    edits: Vec<EditBox>,
    rotation: Rotation,
    include_entities: bool,
    /// The description shown, or the invalid-id message.
    info: Text,
    /// `TestInstanceBlockEntity.errorMessage`, as `setStatus` needs it.
    error: Option<Text>,
}

fn exact_size(count: usize) -> f32 {
    (WIDTH - (count as f32 - 1.0) * PADDING) / count as f32
}

fn widget_size(count: usize) -> f32 {
    exact_size(count).trunc()
}

/// `widgetX`.
fn widget_x(mid: f32, index: usize, count: usize) -> f32 {
    (mid - 158.0 + index as f32 * (PADDING + exact_size(count))).trunc()
}

/// `parseSize`: clamped to 1..=48, 1 when it is not a number.
fn parse_size(value: &str) -> i32 {
    value.parse::<i32>().map_or(1, |v| v.clamp(1, 48))
}

/// `Identifier.tryParse`.
fn parse_id(value: &str) -> Option<String> {
    horizon_client::structure_block::parse_identifier(value)
}

fn rotation_display(rotation: Rotation) -> Text {
    Text::literal(match rotation {
        Rotation::None => "0",
        Rotation::Clockwise90 => "90",
        Rotation::Clockwise180 => "180",
        Rotation::Counterclockwise90 => "270",
    })
}

impl TestInstanceEditor {
    pub(super) fn new(b: &TestInstanceBlock, rotation: Rotation) -> Self {
        let edit = |max: usize, value: String| {
            let mut e = EditBox::new(0.0, 0.0, 80.0, 20.0);
            e.max_length = max;
            e.set_value(&value);
            e.focused = false;
            e
        };
        let mut edits = vec![edit(128, b.data.test.clone().unwrap_or_default())];
        edits.extend(b.data.size.map(|v| edit(15, v.to_string())));
        Self {
            edits,
            rotation,
            include_entities: !b.data.ignore_entities,
            info: Text::empty(),
            error: None,
        }
    }

    fn size(&self) -> [i32; 3] {
        [0, 1, 2].map(|i| parse_size(&self.edits[SIZE + i].value))
    }

    fn save_allowed(&self) -> bool {
        self.rotation == Rotation::None && parse_id(&self.edits[ID].value).is_some()
    }

    /// `updateTestInfo`, local-only: 1.21.4 has no test-instance-block
    /// action packet, so the screen cannot ask the server for the test's
    /// description — it only says when the id is wrong.
    fn update_info(&mut self, _pos: Pos, _init: bool) -> Vec<S> {
        if parse_id(&self.edits[ID].value).is_none() {
            self.info =
                Text::translate("test_instance.description.invalid_id", &[]).with_color(text::RED);
        }
        Vec::new()
    }
}

impl Editor for TestInstanceEditor {
    fn edits(&mut self) -> &mut [EditBox] {
        &mut self.edits
    }

    fn items(&self, _entity: &BlockEntity, w: f32, _h: f32) -> Vec<Item> {
        let mid = (w / 2.0).floor();
        let left = mid - 158.0;
        let tr = |key: &str| Text::translate(key, &[]);
        let button = |id: u8, x: f32, bw: f32, y: f32, text: Text, active: bool| Item::Button {
            id,
            x,
            y,
            w: bw,
            label: text,
            active,
            action: footer_action(id, DONE, CANCEL),
        };
        let label = |key: &str, x: f32| Item::Label {
            text: tr(key),
            x,
            y: if key == "test_instance_block.test_id" {
                30.0
            } else {
                150.0
            },
            right: false,
        };
        let mut items = vec![
            Item::Edit {
                index: ID,
                x: left,
                y: 40.0,
                w: WIDTH,
            },
            Item::Info {
                text: self.info.clone(),
                x: left,
                y: 70.0,
                w: WIDTH,
                h: 72.0,
            },
        ];
        for i in 0..3 {
            items.push(Item::Edit {
                index: SIZE + i,
                x: widget_x(mid, i, 5),
                y: 160.0,
                w: widget_size(5),
            });
        }
        items.push(button(
            ROTATION,
            widget_x(mid, 3, 5),
            widget_size(5),
            160.0,
            rotation_display(self.rotation),
            true,
        ));
        items.push(button(
            ENTITIES,
            widget_x(mid, 4, 5),
            widget_size(5),
            160.0,
            on_off(self.include_entities),
            true,
        ));
        // The export button is for development builds only.
        let save_ok = self.save_allowed();
        items.push(button(
            RESET,
            widget_x(mid, 0, 2),
            widget_size(2),
            185.0,
            tr("test_instance.action.reset"),
            true,
        ));
        items.push(button(
            SAVE,
            widget_x(mid, 1, 2),
            widget_size(2),
            185.0,
            tr("test_instance.action.save"),
            save_ok,
        ));
        items.push(button(
            RUN,
            widget_x(mid, 0, 3),
            widget_size(3),
            210.0,
            tr("test_instance.action.run"),
            true,
        ));
        items.push(button(
            DONE,
            widget_x(mid, 1, 3),
            widget_size(3),
            210.0,
            tr("gui.done"),
            true,
        ));
        items.push(button(
            CANCEL,
            widget_x(mid, 2, 3),
            widget_size(3),
            210.0,
            tr("gui.cancel"),
            true,
        ));
        items.push(Item::Title {
            text: tr("block.minecraft.test_instance_block"),
            y: 10.0,
        });
        items.push(label("test_instance_block.test_id", left));
        items.push(label("test_instance_block.size", left));
        items.push(label("test_instance_block.rotation", widget_x(mid, 3, 5)));
        items.push(label("test_instance_block.entities", widget_x(mid, 4, 5)));
        items
    }

    fn press(
        &mut self,
        entity: &mut BlockEntity,
        pos: Pos,
        id: u8,
        shift: bool,
        _alt: bool,
    ) -> Outcome {
        let delta = if shift { -1 } else { 1 };
        match id {
            // Reset, save and run need the server's test runner, and
            // 1.21.4 has no packet to ask it with: the buttons do nothing.
            RESET | SAVE | RUN => {}
            DONE => return self.done(entity, pos),
            CANCEL => return Outcome::Cancel,
            ROTATION => self.rotation = cycle(&Rotation::ALL, self.rotation, delta),
            ENTITIES => self.include_entities = !self.include_entities,
            _ => {}
        }
        Outcome::Stay
    }

    /// `onDone`: the SET action, applied to the block entity directly.
    /// 1.21.4 has no test-instance-block action packet, so the edit is
    /// local-only and no guest sees it.
    fn done(&mut self, entity: &mut BlockEntity, _pos: Pos) -> Outcome {
        if let BlockEntity::TestInstanceBlock(b) = entity {
            b.data.test = parse_id(&self.edits[ID].value);
            b.data.size = self.size();
            b.data.rotation = self.rotation;
            b.data.ignore_entities = !self.include_entities;
        }
        Outcome::Close
    }

    fn cancel(&mut self, _entity: &mut BlockEntity) {}

    /// The screen has no confirm key of its own.
    fn enter_confirms(&self) -> bool {
        false
    }

    fn opened(&mut self, entity: &BlockEntity, pos: Pos) -> Vec<S> {
        if let BlockEntity::TestInstanceBlock(b) = entity {
            self.error = b
                .data
                .error_message
                .as_ref()
                .map(super::remote::remote_text);
        }
        self.update_info(pos, true)
    }

    fn edited(&mut self, edit: usize, pos: Pos) -> Vec<S> {
        if edit == ID {
            self.update_info(pos, false)
        } else {
            Vec::new()
        }
    }
}

impl Game<'_> {
    /// `LocalPlayer.openTestInstanceBlock`.
    pub(super) fn open_test_instance_block(&mut self, pos: Pos) {
        let Some(BlockEntity::TestInstanceBlock(b)) = self.world.block_entities.get(&pos) else {
            return;
        };
        let editor = TestInstanceEditor::new(b, self.test_rotation(pos));
        self.open_block_editor(pos, Box::new(editor));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> TestInstanceEditor {
        let mut b = TestInstanceBlock::default();
        b.data.test = Some("minecraft:always_pass".into());
        b.data.size = [3, 4, 5];
        TestInstanceEditor::new(&b, Rotation::None)
    }

    #[test]
    fn widgets_are_laid_out_like_the_screen() {
        // 316 wide, 8 apart: two buttons of 154, three of 100, five of 56.
        assert_eq!(widget_size(2), 154.0);
        assert_eq!(widget_size(3), 100.0);
        assert_eq!(widget_size(5), 56.0);
        assert_eq!(widget_x(160.0, 1, 3), 2.0 + 108.0);
    }

    #[test]
    fn sizes_clamp_and_ids_gate_saving() {
        assert_eq!(parse_size("99"), 48);
        assert_eq!(parse_size("0"), 1);
        assert_eq!(parse_size("x"), 1);
        let mut e = editor();
        assert!(e.save_allowed());
        e.rotation = Rotation::Clockwise90;
        assert!(!e.save_allowed());
        e.rotation = Rotation::None;
        e.edits[ID].set_value("Bad Id");
        assert!(!e.save_allowed());
        // An invalid id says so, without asking the server.
        let packets = e.update_info((0, 0, 0), false);
        assert!(packets.is_empty());
        assert!(!e.info.plain().is_empty());
    }

    #[test]
    fn done_applies_the_screens_values_locally() {
        let mut e = editor();
        let mut entity = BlockEntity::TestInstanceBlock(Box::default());
        e.press(&mut entity, (0, 0, 0), ENTITIES, false, false);
        e.press(&mut entity, (0, 0, 0), ROTATION, false, false);
        e.edits[SIZE].set_value("60");
        assert!(matches!(e.done(&mut entity, (1, 2, 3)), Outcome::Close));
        let BlockEntity::TestInstanceBlock(b) = &entity else {
            panic!("the entity is still a test instance block");
        };
        assert_eq!(b.data.test.as_deref(), Some("minecraft:always_pass"));
        assert_eq!(b.data.size, [48, 4, 5]);
        assert!(b.data.ignore_entities);
        assert_eq!(b.data.rotation, Rotation::Clockwise90);
        // Reset, save and run have no local form: they stay on the screen.
        assert!(matches!(
            e.press(&mut entity, (1, 2, 3), RESET, false, false),
            Outcome::Stay
        ));
    }
}
