//! The Statistics screen (`StatsScreen`): the title header, the General,
//! Items and Mobs lists over the player's statistics, and the three list
//! buttons over the Done button in the footer.
use super::{Game, Screen};
use horizon_client::{hud::HudBatch, items::ItemStack, text::Text, ui};
use std::fmt::Write as _;

/// `Stats.CUSTOM` in registration order with its formatter.
const CUSTOM: &[(&str, Format)] = &[
    ("leave_game", Format::Default),
    ("play_time", Format::Time),
    ("total_world_time", Format::Time),
    ("time_since_death", Format::Time),
    ("time_since_rest", Format::Time),
    ("sneak_time", Format::Time),
    ("walk_one_cm", Format::Distance),
    ("crouch_one_cm", Format::Distance),
    ("sprint_one_cm", Format::Distance),
    ("walk_on_water_one_cm", Format::Distance),
    ("fall_one_cm", Format::Distance),
    ("climb_one_cm", Format::Distance),
    ("fly_one_cm", Format::Distance),
    ("walk_under_water_one_cm", Format::Distance),
    ("minecart_one_cm", Format::Distance),
    ("boat_one_cm", Format::Distance),
    ("pig_one_cm", Format::Distance),
    ("happy_ghast_one_cm", Format::Distance),
    ("horse_one_cm", Format::Distance),
    ("aviate_one_cm", Format::Distance),
    ("swim_one_cm", Format::Distance),
    ("strider_one_cm", Format::Distance),
    ("nautilus_one_cm", Format::Distance),
    ("jump", Format::Default),
    ("drop", Format::Default),
    ("damage_dealt", Format::Tenths),
    ("damage_dealt_absorbed", Format::Tenths),
    ("damage_dealt_resisted", Format::Tenths),
    ("damage_taken", Format::Tenths),
    ("damage_blocked_by_shield", Format::Tenths),
    ("damage_absorbed", Format::Tenths),
    ("damage_resisted", Format::Tenths),
    ("deaths", Format::Default),
    ("mob_kills", Format::Default),
    ("animals_bred", Format::Default),
    ("player_kills", Format::Default),
    ("fish_caught", Format::Default),
    ("talked_to_villager", Format::Default),
    ("traded_with_villager", Format::Default),
    ("eat_cake_slice", Format::Default),
    ("fill_cauldron", Format::Default),
    ("use_cauldron", Format::Default),
    ("clean_armor", Format::Default),
    ("clean_banner", Format::Default),
    ("clean_shulker_box", Format::Default),
    ("interact_with_brewingstand", Format::Default),
    ("interact_with_beacon", Format::Default),
    ("inspect_dropper", Format::Default),
    ("inspect_hopper", Format::Default),
    ("inspect_dispenser", Format::Default),
    ("play_noteblock", Format::Default),
    ("tune_noteblock", Format::Default),
    ("pot_flower", Format::Default),
    ("trigger_trapped_chest", Format::Default),
    ("open_enderchest", Format::Default),
    ("enchant_item", Format::Default),
    ("play_record", Format::Default),
    ("interact_with_furnace", Format::Default),
    ("interact_with_crafting_table", Format::Default),
    ("open_chest", Format::Default),
    ("sleep_in_bed", Format::Default),
    ("sleep_in_straw_bed", Format::Default),
    ("open_shulker_box", Format::Default),
    ("open_barrel", Format::Default),
    ("interact_with_blast_furnace", Format::Default),
    ("interact_with_smoker", Format::Default),
    ("interact_with_lectern", Format::Default),
    ("interact_with_campfire", Format::Default),
    ("interact_with_cartography_table", Format::Default),
    ("interact_with_loom", Format::Default),
    ("interact_with_stonecutter", Format::Default),
    ("bell_ring", Format::Default),
    ("raid_trigger", Format::Default),
    ("raid_win", Format::Default),
    ("interact_with_anvil", Format::Default),
    ("interact_with_grindstone", Format::Default),
    ("target_hit", Format::Default),
    ("interact_with_smithing_table", Format::Default),
];
#[derive(Clone, Copy)]
enum Format {
    Default,
    Tenths,
    Distance,
    Time,
}
fn two(v: f64) -> String {
    format!("{v:.2}")
}
/// `StatFormatter`.
fn format(f: Format, v: i32) -> String {
    match f {
        Format::Default => {
            // NumberFormat.getIntegerInstance(Locale.US): thousands separators.
            let digits = v.unsigned_abs().to_string();
            let mut out = String::new();
            for (i, c) in digits.chars().enumerate() {
                if i > 0 && (digits.len() - i).is_multiple_of(3) {
                    out.push(',');
                }
                out.push(c);
            }
            if v < 0 {
                out.insert(0, '-');
            }
            out
        }
        Format::Tenths => two(f64::from(v) * 0.1),
        Format::Distance => {
            let m = f64::from(v) / 100.0;
            let km = m / 1000.0;
            if km > 0.5 {
                format!("{} km", two(km))
            } else if m > 0.5 {
                format!("{} m", two(m))
            } else {
                format!("{v} cm")
            }
        }
        Format::Time => {
            let s = f64::from(v) / 20.0;
            let (min, h) = (s / 60.0, s / 3600.0);
            let (d, y) = (h / 24.0, h / 24.0 / 365.0);
            if y > 0.5 {
                format!("{} y", two(y))
            } else if d > 0.5 {
                format!("{} d", two(d))
            } else if h > 0.5 {
                format!("{} h", two(h))
            } else if min > 0.5 {
                format!("{} min", two(min))
            } else {
                // Java's double + " s".
                let mut t = String::new();
                let _ = write!(t, "{s:?}");
                format!("{t} s")
            }
        }
    }
}
/// The item columns: (stat type, header sprite).
const COLUMNS: [(&str, &str); 6] = [
    ("mined", "gui/sprites/statistics/block_mined"),
    ("broken", "gui/sprites/statistics/item_broken"),
    ("crafted", "gui/sprites/statistics/item_crafted"),
    ("used", "gui/sprites/statistics/item_used"),
    ("picked_up", "gui/sprites/statistics/item_picked_up"),
    ("dropped", "gui/sprites/statistics/item_dropped"),
];
/// `ItemStatisticsList`'s rows: items with a positive item column, plus
/// `Block.asItem` (`block_item`) of blocks with a positive `mined` count,
/// whose stat is keyed by block; air is dropped.
fn item_rows(stats: &super::stats::Stats, block_item: impl Fn(&str) -> String) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    for (ty, _) in COLUMNS {
        for (k, v) in stats
            .values
            .get(&format!("minecraft:{ty}"))
            .into_iter()
            .flatten()
        {
            let item = if ty == "mined" {
                block_item(k)
            } else {
                k.clone()
            };
            if *v > 0 && item != "minecraft:air" && !items.contains(&item) {
                items.push(item);
            }
        }
    }
    items
}
/// `ItemRow`'s value in column `ty`: `mined` is the count of a `BlockItem`'s
/// block (`item_block`, `BlockItem.getBlock`), none ("-") for other items.
fn item_stat(
    stats: &super::stats::Stats,
    ty: &str,
    item: &str,
    item_block: impl Fn(&str) -> Option<String>,
) -> Option<i32> {
    if ty == "mined" {
        item_block(item).map(|block| stats.get(ty, &block))
    } else {
        Some(stats.get(ty, item))
    }
}
/// `ItemRowComparator.compare`: by the column's value (-1 for none), equal
/// values by item registry id, the whole order reversed when `order` is -1.
fn sort_rows(rows: &mut [String], order: i32, key: impl Fn(&str) -> (i32, usize)) {
    rows.sort_by_cached_key(|i| key(i));
    if order < 0 {
        rows.reverse();
    }
}
#[derive(Default)]
pub(super) struct StatsUi {
    tab: usize,
    /// The list's scrolling: the wheel and the held scrollbar.
    scroll: horizon_client::ui::Scroll,
    sort: Option<(usize, i32)>,
    /// `ItemStatisticsList.headerPressed`: the sort column held down, cleared
    /// when the button comes up.
    header_pressed: Option<usize>,
}
/// `ItemStatisticsList.clickedHeader`: the column a click at `x` (pixels
/// right of the list's left edge) sorts, or none. A column's window is the
/// 36 pixels left of its `getColumnX` — twice the 18-pixel icon.
fn header_column(x: f32) -> Option<usize> {
    (0..COLUMNS.len()).find(|&c| {
        let col_x = 75.0 + 40.0 * c as f32;
        x >= col_x - 36.0 && x <= col_x
    })
}
/// `ItemStatisticsList.sortByColumn`: a click on a column sorts it
/// descending, then ascending, then off; a click on another column starts
/// it descending.
fn sort_click(sort: Option<(usize, i32)>, c: usize) -> Option<(usize, i32)> {
    match sort {
        Some((sc, -1)) if sc == c => Some((c, 1)),
        Some((sc, _)) if sc == c => None,
        _ => Some((c, -1)),
    }
}
/// `StatsScreen.initButtons`: the footer is a vertical layout of the three
/// list buttons (120 wide, 5 apart) over the Done button, centred in the
/// 58-pixel footer (`FrameLayout` centres its child: (58 - 45) / 2 = 6).
/// The buttons row's y and the Done button's y.
fn stats_footer(h: f32) -> (f32, f32) {
    (h - FOOTER + 6.0, h - FOOTER + 31.0)
}
/// The buttons row's left edge: three 120-pixel buttons 5 apart, centred.
fn stats_buttons_x(w: f32) -> f32 {
    ((w - 370.0) / 2.0).floor()
}
const HEADER: f32 = 33.0;
const FOOTER: f32 = 58.0;
impl Game<'_> {
    pub(super) fn open_stats(&mut self) {
        self.load_stats();
        // StatsScreen.init: a server's client asks for its statistics.
        if self.remote.is_some() {
            self.send(horizon_protocol::packets::play::Serverbound::ClientCommand(
                horizon_protocol::packets::play::ClientCommand { action: 1 },
            ));
        }
        self.stats_ui = StatsUi::default();
        self.screen = Screen::Stats;
    }
    /// The list's scrollbar column (right of the 280 pixel list) and the
    /// pixels it scrolls.
    fn stats_bar(w: f32) -> f32 {
        ((w - 280.0) / 2.0).floor() + 280.0 + 6.0
    }
    fn stats_content(&self) -> f32 {
        let (row_h, rows) = self.stats_rows();
        row_h * rows as f32 + 4.0
    }
    fn stats_rows(&self) -> (f32, usize) {
        match self.stats_ui.tab {
            0 => (14.0, CUSTOM.len()),
            // The +1 is `ItemStatisticsList`'s 22-pixel header (its 6-arg
            // `super` call sets `renderHeader` and `headerHeight`), one
            // row's height, ahead of the rows.
            1 => (22.0, self.stat_items().len() + 1),
            _ => (36.0, self.stat_mobs().len()),
        }
    }
    /// Whether tab `i` can be chosen: the Items and Mobs tabs are active only
    /// while their list has rows.
    fn stats_tab_active(&self, i: usize) -> bool {
        match i {
            1 => !self.stat_items().is_empty(),
            2 => !self.stat_mobs().is_empty(),
            _ => true,
        }
    }
    /// Items (and block items) with any non-zero column.
    fn stat_items(&self) -> Vec<String> {
        let mut items = item_rows(&self.stats, |block| {
            self.pack.assets.registry.default_state(block).map_or_else(
                || "minecraft:air".to_owned(),
                |s| self.res.facts().get(s).item.to_string(),
            )
        });
        if let Some((col, order)) = self.stats_ui.sort {
            let ty = COLUMNS[col].0;
            // `Item.getId`: one pass over the registry, not a scan per row
            // (the screen asks for its rows several times a frame).
            let ids: std::collections::HashMap<&str, usize> = self
                .res
                .items()
                .registry_order
                .iter()
                .enumerate()
                .map(|(n, id)| (&**id, n))
                .collect();
            sort_rows(&mut items, order, |i| {
                let id = ids.get(i).copied().unwrap_or(usize::MAX);
                (self.stat_item(ty, i).unwrap_or(-1), id)
            });
        }
        items
    }
    fn stat_item(&self, ty: &str, item: &str) -> Option<i32> {
        item_stat(&self.stats, ty, item, |i| {
            let state = self.res.items().get(i)?.block?;
            Some(self.res.facts().get(state).name.to_string())
        })
    }
    fn stat_mobs(&self) -> Vec<String> {
        let mut mobs: Vec<String> = Vec::new();
        for ty in ["killed", "killed_by"] {
            if let Some(m) = self.stats.values.get(&format!("minecraft:{ty}")) {
                for (k, v) in m {
                    if *v > 0 && !mobs.contains(k) {
                        mobs.push(k.clone());
                    }
                }
            }
        }
        mobs
    }
    /// `MobRow`: the mob's name, its kills and killed-by lines, and their
    /// colours (grey when none).
    fn mob_row(&self, mob: &str) -> (Text, Text, Text, (u32, u32)) {
        let name = Text::translate(&format!("entity.{}", mob.replace(':', ".")), &[]);
        let kills = self.stats.get("killed", mob);
        let by = self.stats.get("killed_by", mob);
        let k = if kills == 0 {
            Text::translate(
                "stat_type.minecraft.killed.none",
                std::slice::from_ref(&name),
            )
        } else {
            Text::translate(
                "stat_type.minecraft.killed",
                &[Text::literal(kills.to_string()), name.clone()],
            )
        };
        let b = if by == 0 {
            Text::translate(
                "stat_type.minecraft.killed_by.none",
                std::slice::from_ref(&name),
            )
        } else {
            Text::translate(
                "stat_type.minecraft.killed_by",
                &[name.clone(), Text::literal(by.to_string())],
            )
        };
        let colors = (
            if kills > 0 { 0xFFBA_BABA } else { 0xFF80_8080 },
            if by > 0 { 0xFFBA_BABA } else { 0xFF80_8080 },
        );
        (name, k, b, colors)
    }
    pub(super) fn draw_stats(&mut self, batch: &mut HudBatch) {
        horizon_client::widgets::begin_screen(|| {
            horizon_client::widgets::ScreenInfo::new(
                "stats",
                super::screen_info::tr("gui.stats", &[]),
            )
        });
        let (w, h) = (batch.width, batch.height);
        let atlas = &self.gui.atlas;
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, true);
        let (top, bottom) = (HEADER, h - FOOTER);
        ui::separators(batch, atlas, top, bottom, true);
        // The header's title: `HeaderAndFooterLayout` centres it vertically
        // in the 33-pixel header, (33 - 9) / 2 = 12.
        ui::centered_text(batch, atlas, &Text::translate("gui.stats", &[]), 0.0, 12.0, w, u32::MAX);
        // The list.
        let (row_h, rows) = self.stats_rows();
        let content = row_h * rows as f32 + 4.0;
        self.stats_ui.scroll.clamp(top, bottom, content);
        let left = ((w - 280.0) / 2.0).floor();
        let scroll = self.stats_ui.scroll.amount;
        let row_y = |i: usize| top + 4.0 + i as f32 * row_h - scroll;
        let visible = |y: f32| y >= top && y + row_h <= bottom;
        // The list's row under the mouse (`getHovered`).
        let hovered_row = {
            let (cx, cy) = self.cursor;
            let i = ((cy - (top + 4.0) + scroll) / row_h).floor();
            let inside = cy >= top && cy < bottom && cx >= left && cx < left + 280.0;
            (inside && i >= 0.0 && (i as usize) < rows).then_some(i as usize)
        };
        let list_rect = [0.0, top, w, bottom - top];
        let list_key = horizon_client::widgets::key_of(&("stats/list", self.stats_ui.tab));
        let tr = super::screen_info::tr;
        // GeneralStatisticsList and MobsStatisticsList: ObjectSelectionLists
        // narrating the hovered row (none is selected).
        let object_list = |narration: &dyn Fn(usize) -> String| {
            horizon_client::widgets::narratable(|| {
                use horizon_client::widgets::{ListKind, Narratable, Row};
                let hovered = hovered_row.map(|i| Row::new(i, narration(i)));
                Narratable::list(ListKind::Object, rows, hovered, None, list_rect).key(list_key)
            });
        };
        // The hovered item's tooltip, drawn over the list.
        let mut tip = None;
        match self.stats_ui.tab {
            0 => {
                let mut sorted: Vec<(Text, String)> = CUSTOM
                    .iter()
                    .map(|(k, f)| {
                        let label = Text::translate(&format!("stat.minecraft.{k}"), &[]);
                        let v = self.stats.get("custom", &format!("minecraft:{k}"));
                        (label, format(*f, v))
                    })
                    .collect();
                sorted.sort_by_key(|(l, _)| l.plain());
                object_list(&|i| {
                    let (label, value) = &sorted[i];
                    tr("narrator.select", &[&format!("{} {value}", label.plain())])
                });
                for (i, (label, value)) in sorted.iter().enumerate() {
                    let y = row_y(i);
                    if !visible(y) {
                        continue;
                    }
                    let color = if i % 2 == 0 { u32::MAX } else { 0xFFBA_BABA };
                    let ty = y + (row_h - 8.0) / 2.0;
                    batch.text_styled(atlas, label, left + 2.0, ty.floor(), color, true);
                    let vw = HudBatch::text_width(atlas, value);
                    batch.text(
                        atlas,
                        value,
                        left + 280.0 - vw - 4.0,
                        ty.floor(),
                        color,
                        true,
                    );
                }
            }
            1 => {
                let items = self.stat_items();
                if items.is_empty() {
                    return self.draw_stats_footer(batch, w, h);
                }
                // ItemStatisticsList, an ObjectSelectionList over its
                // ItemRows: the header is not an entry, so the hovered row
                // is one past the hovered item, and a row narrates
                // `narrator.select` with the item's name.
                let hovered_item = hovered_row.and_then(|p| p.checked_sub(1));
                horizon_client::widgets::narratable(|| {
                    use horizon_client::widgets::{ListKind, Narratable, Row};
                    let hovered = hovered_item.map(|i| {
                        let stack = ItemStack::new(&items[i], 1);
                        let name =
                            horizon_client::stations::hover_name(&self.pack.names, &stack);
                        Row::new(i, tr("narrator.select", &[&name]))
                    });
                    Narratable::list(ListKind::Object, items.len(), hovered, None, list_rect)
                        .key(list_key)
                });
                // The header, drawn by `renderHeader` above the rows and
                // scrolling with them: a column's background is the slot
                // sprite while it is held, the header sprite otherwise, its
                // icon shifted a pixel while held, and the sort arrow 18
                // pixels left of the icon.
                let y = row_y(0);
                if visible(y) {
                    for (c, (_, sprite)) in COLUMNS.iter().enumerate() {
                        let x = left + 75.0 + 40.0 * c as f32 - 18.0;
                        let pressed = self.stats_ui.header_pressed == Some(c);
                        let (dx, bg) = if pressed {
                            (1.0, "gui/sprites/container/slot")
                        } else {
                            (0.0, "gui/sprites/statistics/header")
                        };
                        batch.sprite(atlas, bg, x, y + 1.0);
                        batch.sprite(atlas, sprite, x + dx, y + 1.0 + dx);
                        if let Some((sc, order)) = self.stats_ui.sort
                            && sc == c
                        {
                            let s = if order < 0 {
                                "gui/sprites/statistics/sort_down"
                            } else {
                                "gui/sprites/statistics/sort_up"
                            };
                            batch.sprite(atlas, s, x - 18.0, y + 1.0);
                        }
                    }
                }
                for (i, item) in items.iter().enumerate() {
                    let y = row_y(i + 1);
                    if !visible(y) {
                        continue;
                    }
                    let stack = ItemStack::new(item, 1);
                    batch.sprite(
                        &self.gui.atlas,
                        "gui/sprites/container/slot",
                        left + 1.0,
                        y + 1.0,
                    );
                    self.draw_item(batch, &stack, left + 2.0, y + 2.0);
                    // `renderDecorations`: hovering a row's slot shows the
                    // item's name at the cursor.
                    if ui::hovered(self.cursor, left + 1.0, y + 1.0, 18.0, 18.0) {
                        tip = Some(Text::literal(horizon_client::stations::hover_name(
                            &self.pack.names,
                            &stack,
                        )));
                    }
                    let atlas = &self.gui.atlas;
                    let color = if i % 2 == 0 { u32::MAX } else { 0xFFBA_BABA };
                    for (c, (ty, _)) in COLUMNS.iter().enumerate() {
                        let x = left + 75.0 + 40.0 * c as f32;
                        let text = match self.stat_item(ty, item) {
                            Some(v) => format(Format::Default, v),
                            None => Text::translate("stats.none", &[]).plain(),
                        };
                        let tw = HudBatch::text_width(atlas, &text);
                        batch.text(atlas, &text, x - tw, (y + 7.0).floor(), color, true);
                    }
                }
                // `renderDecorations`: with no row under the mouse, hovering
                // a column shows its stat type's name.
                if hovered_item.is_none() && self.cursor.1 >= top && self.cursor.1 < bottom {
                    let x = self.cursor.0 - left;
                    if let Some(c) = (0..COLUMNS.len()).find(|&c| {
                        let col_x = 75.0 + 40.0 * c as f32;
                        x >= col_x - 18.0 && x <= col_x
                    }) {
                        tip = Some(Text::translate(
                            &format!("stat_type.minecraft.{}", COLUMNS[c].0),
                            &[],
                        ));
                    }
                }
            }
            _ => {
                let mobs = self.stat_mobs();
                object_list(&|i| {
                    let (_, kills, killed_by, _) = self.mob_row(&mobs[i]);
                    let both = horizon_client::widgets::join_for_narration(&[
                        &kills.plain(),
                        &killed_by.plain(),
                    ]);
                    tr("narrator.select", &[&both])
                });
                for (i, mob) in mobs.iter().enumerate() {
                    let y = row_y(i);
                    if !visible(y) {
                        continue;
                    }
                    let (name, k, b, (ck, cb)) = self.mob_row(mob);
                    batch.text_styled(atlas, &name, left + 2.0, y + 1.0, u32::MAX, true);
                    batch.text_styled(atlas, &k, left + 12.0, y + 10.0, ck, true);
                    batch.text_styled(atlas, &b, left + 12.0, y + 19.0, cb, true);
                }
            }
        }
        let atlas = &self.gui.atlas;
        ui::scrollbar(
            batch,
            atlas,
            Self::stats_bar(w),
            top,
            bottom,
            content,
            self.stats_ui.scroll.amount,
        );
        self.draw_stats_footer(batch, w, h);
        if let Some(tip) = tip {
            let atlas = &self.gui.atlas;
            ui::tooltip(batch, atlas, std::slice::from_ref(&tip), self.cursor);
        }
    }
    fn draw_stats_footer(&self, batch: &mut HudBatch, w: f32, h: f32) {
        let atlas = &self.gui.atlas;
        // `initButtons`: the three list buttons, each active only while its
        // list has rows, over the Done button.
        const TABS: [&str; 3] = ["stat.generalButton", "stat.itemsButton", "stat.mobsButton"];
        let (buttons_y, done_y) = stats_footer(h);
        let x0 = stats_buttons_x(w);
        for (i, key) in TABS.into_iter().enumerate() {
            let x = x0 + 125.0 * i as f32;
            let active = self.stats_tab_active(i);
            let hover = ui::hovered(self.cursor, x, buttons_y, 120.0, 20.0);
            ui::button(
                batch,
                atlas,
                x,
                buttons_y,
                120.0,
                20.0,
                &Text::translate(key, &[]),
                active,
                hover,
            );
        }
        let bx = (w / 2.0 - 100.0).floor();
        let hover = ui::hovered(self.cursor, bx, done_y, 200.0, 20.0);
        ui::action_button(
            batch,
            atlas,
            horizon_client::widgets::Action::Done,
            bx,
            done_y,
            200.0,
            20.0,
            &Text::translate("gui.done", &[]),
            true,
            hover,
        );
    }
    pub(super) fn stats_click(&mut self, size: [u32; 2]) {
        // A press on the scrollbar holds it and clicks nothing else.
        if self.stats_press_scrollbar(size) {
            return;
        }
        let b = HudBatch::new(size);
        let (w, h) = (b.width, b.height);
        let (buttons_y, done_y) = stats_footer(h);
        let x0 = stats_buttons_x(w);
        for i in 0..3 {
            if ui::hovered(self.cursor, x0 + 125.0 * i as f32, buttons_y, 120.0, 20.0)
                && self.stats_tab_active(i)
            {
                // A Button plays its click on any press, even the current
                // tab's; only a switch re-lays the list out at the top.
                self.audio
                    .play("minecraft:ui.button.click", None, 0.25, 1.0);
                if self.stats_ui.tab != i {
                    self.stats_ui.tab = i;
                    self.stats_ui.scroll.amount = 0.0;
                }
                return;
            }
        }
        if ui::hovered(
            self.cursor,
            (w / 2.0 - 100.0).floor(),
            done_y,
            200.0,
            20.0,
        ) {
            self.audio
                .play("minecraft:ui.button.click", None, 0.25, 1.0);
            self.screen = Screen::Paused;
            return;
        }
        if self.stats_ui.tab == 1 {
            // `clickedHeader` ignores the click's y: any click in the list
            // whose x falls in a column's window sorts, and holds the
            // column down until the button comes up.
            let left = ((w - 280.0) / 2.0).floor();
            let top = HEADER;
            let bottom = h - FOOTER;
            if self.cursor.1 >= top
                && self.cursor.1 < bottom
                && let Some(c) = header_column(self.cursor.0 - left)
            {
                self.stats_ui.header_pressed = Some(c);
                self.stats_ui.sort = sort_click(self.stats_ui.sort, c);
                self.audio
                    .play("minecraft:ui.button.click", None, 0.25, 1.0);
            }
        }
    }
    pub(super) fn stats_scroll(&mut self, amount: f32) {
        let (row_h, _) = self.stats_rows();
        let h = self.gui_size().1;
        let content = self.stats_content();
        self.stats_ui
            .scroll
            .wheel(amount, row_h / 2.0, HEADER, h - FOOTER, content);
    }
    /// A press on the statistics list's scrollbar holds it
    /// (`AbstractSelectionList.updateScrolling`).
    pub(super) fn stats_press_scrollbar(&mut self, size: [u32; 2]) -> bool {
        let b = HudBatch::new(size);
        let (w, h) = (b.width, b.height);
        let content = self.stats_content();
        self.stats_ui
            .scroll
            .press(self.cursor, Self::stats_bar(w), HEADER, h - FOOTER, content)
    }
    /// The pointer moved while the scrollbar is held.
    pub(super) fn stats_drag(&mut self) {
        if !self.stats_ui.scroll.held() {
            return;
        }
        let h = self.gui_size().1;
        let content = self.stats_content();
        self.stats_ui
            .scroll
            .drag(self.cursor.1, HEADER, h - FOOTER, content);
    }
    /// The button came up: the scrollbar lets go, and a held sort column
    /// is released (`headerPressed` clears when the button is not pressed).
    pub(super) fn stats_release(&mut self) {
        self.stats_ui.scroll.release();
        self.stats_ui.header_pressed = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ItemStatisticsList`: a mined block's row is its item (`Block.asItem`),
    /// air dropped, and a `BlockItem`'s Mined column reads its block's count.
    /// The block <-> item pairs are vanilla 26.3's (`Items`: wheat seeds place
    /// wheat, redstone places redstone wire, a torch places torch and wall
    /// torch; water has no item).
    #[test]
    fn mined_blocks_count_under_the_item_that_places_them() {
        let mut stats = super::super::stats::Stats::default();
        stats.add("mined", "minecraft:wheat", 3);
        stats.add("mined", "minecraft:redstone_wire", 5);
        stats.add("mined", "minecraft:wall_torch", 7);
        stats.add("mined", "minecraft:torch", 2);
        stats.add("mined", "minecraft:water", 4);
        stats.add("used", "minecraft:wheat_seeds", 1);
        stats.add("crafted", "minecraft:stick", 6);
        let block_item = |b: &str| {
            match b {
                "minecraft:wheat" => "minecraft:wheat_seeds",
                "minecraft:redstone_wire" => "minecraft:redstone",
                "minecraft:wall_torch" | "minecraft:torch" => "minecraft:torch",
                _ => "minecraft:air",
            }
            .to_owned()
        };
        let item_block = |i: &str| {
            match i {
                "minecraft:wheat_seeds" => Some("minecraft:wheat"),
                "minecraft:redstone" => Some("minecraft:redstone_wire"),
                "minecraft:torch" => Some("minecraft:torch"),
                _ => None,
            }
            .map(str::to_owned)
        };
        let mut rows = item_rows(&stats, block_item);
        rows.sort();
        assert_eq!(
            rows,
            [
                "minecraft:redstone",
                "minecraft:stick",
                "minecraft:torch",
                "minecraft:wheat_seeds"
            ]
        );
        let mined = |i: &str| item_stat(&stats, "mined", i, item_block);
        assert_eq!(mined("minecraft:wheat_seeds"), Some(3));
        assert_eq!(mined("minecraft:redstone"), Some(5));
        assert_eq!(
            mined("minecraft:torch"),
            Some(2),
            "the torch block's own count"
        );
        assert_eq!(mined("minecraft:stick"), None, "not a block item: '-'");
        assert_eq!(
            item_stat(&stats, "used", "minecraft:wheat_seeds", item_block),
            Some(1)
        );
    }

    /// `ItemRowComparator` with `sortByColumn`'s first click (order -1): the
    /// biggest value first, "-" (-1) last, equal values by item registry id,
    /// that order reversed too; the second click (order 1) is the mirror.
    #[test]
    fn item_rows_sort_by_value_then_registry_id() {
        let rows = || ["c", "a", "none", "b", "d"].map(str::to_owned);
        // (value, registry id)
        let key = |i: &str| match i {
            "a" => (5, 0),
            "b" => (2, 1),
            "c" => (5, 2),
            "d" => (0, 3),
            _ => (-1, 4),
        };
        let mut down = rows();
        sort_rows(&mut down, -1, key);
        assert_eq!(down, ["c", "a", "b", "d", "none"]);
        let mut up = rows();
        sort_rows(&mut up, 1, key);
        assert_eq!(up, ["none", "d", "b", "a", "c"]);
    }

    /// `ItemStatisticsList.clickedHeader`: a column's click window is the 36
    /// pixels left of its `getColumnX` — column 0 spans 39..75, column 1
    /// spans 79..115, and the gap between them sorts nothing.
    #[test]
    fn header_click_windows_are_36_wide() {
        assert_eq!(header_column(39.0), Some(0));
        assert_eq!(header_column(75.0), Some(0));
        assert_eq!(header_column(38.0), None);
        assert_eq!(header_column(76.0), None);
        assert_eq!(header_column(79.0), Some(1));
        assert_eq!(header_column(115.0), Some(1));
        assert_eq!(header_column(275.0), Some(5));
        assert_eq!(header_column(276.0), None);
    }

    /// `ItemStatisticsList.sortByColumn`: the first click sorts descending,
    /// the second ascending, the third off; a click on another column starts
    /// it descending.
    #[test]
    fn sort_clicks_cycle_descending_ascending_off() {
        let mut sort = None;
        sort = sort_click(sort, 2);
        assert_eq!(sort, Some((2, -1)));
        sort = sort_click(sort, 2);
        assert_eq!(sort, Some((2, 1)));
        sort = sort_click(sort, 2);
        assert_eq!(sort, None);
        sort = sort_click(sort, 4);
        assert_eq!(sort, Some((4, -1)));
        sort = sort_click(sort, 1);
        assert_eq!(sort, Some((1, -1)));
    }
}
