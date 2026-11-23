# Item 16: block_entity_render.rs (33 classes, 219 methods, 20 named)

Diff: `javapdiff/16/diff.txt` (8,286 lines, +3468/-3404).
Sections: Font$PreparedTextBuilder 1, BakedSheetGlyph 132/245,
LayerDefinitions 335, BellModel$State 4142, BookModel 4173/4259,
CopperGolemStatueModel 4372, AbstractEndPortalRenderer 4550,
AbstractSignRenderer 4804, BeaconRenderer 5342, BellRenderer 5381,
BrightnessCombiner 5487, ChestRenderer 5531, ConduitRenderer 6058,
CopperGolemStatueBlockRenderer 6476, EnchantTableRenderer 6679,
HangingSignRenderer 6820, LecternRenderer 7311, ShulkerBoxRenderer 7446,
SkullBlockRenderer 7724, TheEndGatewayRenderer 8067,
TheEndPortalRenderer 8157, TextFeatureRenderer 8232,
DoubleBlockCombiner 8276. Plus the new 1.21.11-only SignRenderer
(dumped to /tmp/signrenderer11.txt) and the removed 26.1-only
StandingSignRenderer (+$1, +$Models), HangingSignRenderer$1/$Models,
BannerRenderer$1.

ALL HUNKS READ 2026-11-19 evening. Triage scripts /tmp/const16.py,
call16.py, branch16.py (from the item-15 scripts, s/15/16/).

## Ports

### A. Texture paths (4 moves, 10 sites, jar-verified in the 1.21.11 client jar)
- `entity/end_portal/end_portal` -> `entity/end_portal`
- `entity/beacon/beacon_beam` -> `entity/beacon_beam`
- `entity/end_portal/end_gateway_beam` -> `entity/end_gateway_beam`
- `entity/enchantment/enchanting_table_book` -> `entity/enchanting_table_book`
Sites: block_entity_render.rs (3), block_entity_render_fixture.rs (2),
game.rs, game/beacon.rs, game/win.rs, game/world_primitives.rs,
item_render/special.rs.

### B. Chest Christmas precedence (ChestRenderer.getChestMaterial)
26.1 checked copper FIRST (copper chests never got christmas textures);
1.21.11 checks ender -> xmas -> trapped -> copper, so copper chests
wear the christmas textures on 24-26 December and only ender chests
keep theirs. Both xmas sites in block_entity_render.rs (the sprite
collection and the draw) now match the four copper chest blocks too
(`copper_chest`, `exposed_`, `weathered_`, `oxidized_`; waxed variants
are unwaxed by `classify` before matching). The christmas single/left/
right selection by part is unchanged; copper chests still draw
vanilla's chest model (no CEM name).

## Comment-only
- Sign text: `StandingSignRenderer.textTransformation` ->
  `SignRenderer.translateSign + AbstractSignRenderer.translateSignText`;
  `HangingSignRenderer.textTransformation` -> `translateBase +
  translateSignText`; the Kind doc cites `SignRenderer`.
- End portal: `TheEndPortalRenderer.TRANSFORMATION` ->
  `AbstractEndPortalRenderer.getOffsetDown/getOffsetUp`;
  `FACES`/`submitCube` -> `renderCube`/`renderFace`.
- Lectern: `BOOK_STATE = forAnimation(0, 0.1, 0.9, 1.2)` ->
  `bookState = new State(0, 0.1, 0.9, 1.2)`.
- block_entity_anim.rs: `BookModel.State.forAnimation` ->
  `BookModel.State` (animationPos, pageFlip1, pageFlip2, open).

## Verified no-ports (all jar/diff-checked)
- Font$PreparedTextBuilder, BakedSheetGlyph (+$GlyphInstance): field
  order / access / type noise only.
- LayerDefinitions: the Adult/Baby model-class merge (other items'
  territory); the only block-entity change is
  `StandingSignRenderer.createSignLayer` -> `SignRenderer.createSignLayer`,
  byte-identical geometry (sign box -12,-14,-1 24x12x2; stick -1,-2,-1
  2x14x2; 64x32). The Rust's sign boards are block models anyway.
- BookModel/$State: the openness formula moved from `forAnimation`
  into `setupAnim` with the same values; the Rust
  (block_entity_anim.rs:228) already computes
  `((time * 0.02).sin() * 0.1 + 1.25) * open`.
- BellModel$State: access-modifier noise. BellRenderer: SpriteId ->
  Material churn only.
- CopperGolemStatueModel/BlockRenderer: the y-rot moved from the
  renderer into `setupAnim(Direction)` with a sign flip that the
  zRot=PI conjugation exactly cancels (Rz(PI).Ry(+th) = Ry(-th).Rz(PI));
  the Rust's `T.Ry(-th)` matrix + root zRot=PI matches. `COPPER_GOLEM_POSE`
  is the same property name, only the Java field moved.
- End portal: 26.1's portal TRANSFORMATION (translate y 0.375, scale y
  0.375) equals 1.21.11's getOffsetDown/Up (0.375/0.75); the gateway's
  0/1 overrides equal 26.1's full cube. `shouldRenderFace` is identical
  in both versions (Y axis only), so the side faces — full-height in
  1.21.11's renderCube, squashed in 26.1 — are never drawn for portals.
  PORTAL_FACES winding is preserved (1.21.11's renderFace vertex order
  is the same cycle, rotated by one vertex).
- Signs: every transform value identical — standing/wall 2/3 scale
  (text 0.010416667), hanging text 0.9 (0.0140625), offsets (0, 1/3,
  0.046666667) / (0, -0.32, 0.073), wall (0, -0.3125, -0.4375),
  hanging base (0.5, 0.9375, 0.5) + (0, -0.3125, 0); rotation via
  `RotationSegment.convertToDegrees` / `Direction.toYRot()` in both
  (26.1's WallAndGroundTransformations lambdas, 1.21.11's
  `SignBlock.getYRotationDegrees`). Only the structure changed
  (precomputed SignTransformations -> translateSign/translateSignText).
  Text logic identical (getDarkColor, 4 lines x textLineHeight/2,
  glowing -> 15728880, outline rules).
- BeaconRenderer: texture path only (ported above).
- BrightnessCombiner: 1.21.11 inlines `LightCoordsUtil.max` exactly
  (max of block parts, max of sky parts, packed) — verified against
  the 26.1 jar.
- ChestRenderer: transform identical (`rotationAround(-toYRot,
  0.5, 0, 0.5)` == translate(0.5,0.5,0.5).rotY(-angle).translate(-0.5)
  for a Y rotation); lid easing identical (`1-(1-open)^3`);
  `xmasTextures` identical (`SpecialDates.isExtendedChristmas`).
- ConduitRenderer: submit constants and calls identical; only the
  `ConduitRenderState` -> `CondiutRenderState` typo rename and the
  SpriteId -> Material move. DEFAULT_TRANSFORMATION (translate
  0.5,0.5,0.5) inlined.
- EnchantTableRenderer: `BookModel$State.forAnimation(...)` -> `new
  BookModel$State(...)`, same four args (time, clamp(flip1),
  clamp(flip2), open).
- LecternRenderer: BOOK_STATE static -> bookState field, same values.
- ShulkerBoxRenderer: `prepareModel` repeats 26.1's
  `createModelTransform` matrix sequence exactly (translate 0.5,
  scale 0.9995^3, direction rotation, scale (1,-1,-1), translate
  (0,-1,0)); progress flows identically (state.progress -> setupAnim).
- SkullBlockRenderer: rotation moved from the PoseStack
  (R_y(-degrees)) into the model state (yRot = +degrees); identical by
  the S.R_y(+th) = R_y(-th).S conjugation with S = (-1,-1,1) (verified
  against SkullModel.setupAnim, identical in both jars:
  head.yRot = state.yRot * DEG_TO_RAD; 26.1's submitSkull left yRot 0).
  Wall rotation: `convertToSegment(opposite)` + `convertToDegrees` ==
  `opposite.toYRot()` for the four horizontal facings. Render type
  `entityCutoutZOffset` -> `entityCutoutNoCullZOffset` matches the
  Rust already: the entity pipeline here never culls (draw_part sets
  no CULL_BACK), so skulls were already two-sided.
- TextFeatureRenderer: `renderTranslucent` -> `render`, Matrix4fc ->
  Matrix4f, pure renames. DoubleBlockCombiner: `is(Object)` ->
  `is(Block)` overload noise.

## Checks
- `cargo check -p horizon-client-render --lib` clean (1m14s, JOBS=1).
- `cargo check -p horizon-client --lib` clean (2m00s, JOBS=1).
- No new unit test: the xmas branch is date-gated
  (`extended_christmas` reads the real date, OnceLock-cached) and has
  no existing test; the change is a match-arm extension.
