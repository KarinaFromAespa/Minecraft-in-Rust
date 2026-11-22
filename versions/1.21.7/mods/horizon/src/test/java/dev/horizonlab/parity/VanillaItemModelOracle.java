package dev.horizonlab.parity;

import com.google.gson.*;
import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.serialization.Codec;
import com.mojang.serialization.JsonOps;
import com.mojang.serialization.MapCodec;
import java.io.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import java.util.stream.*;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.model.geom.EntityModelSet;
import net.minecraft.client.renderer.MultiBufferSource;
import net.minecraft.client.renderer.block.model.BakedQuad;
import net.minecraft.client.renderer.block.model.BlockModel;
import net.minecraft.client.renderer.block.model.ItemModelGenerator;
import net.minecraft.client.renderer.block.model.ItemTransform;
import net.minecraft.client.renderer.item.*;
import net.minecraft.client.renderer.item.properties.select.SelectItemModelProperty;
import net.minecraft.client.renderer.special.SpecialModelRenderer;
import net.minecraft.client.renderer.special.SpecialModelRenderers;
import net.minecraft.client.renderer.texture.*;
import net.minecraft.client.resources.metadata.animation.AnimationMetadataSection;
import net.minecraft.client.resources.model.*;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.component.DataComponentType;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.resources.RegistryOps;
import net.minecraft.util.StrictJsonParser;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.item.*;
import net.minecraft.world.item.component.BundleContents;
import net.minecraft.world.level.GrassColor;
import net.minecraft.world.level.Level;
import org.joml.Matrix4f;
import org.joml.Vector3f;

/**
 * Drives the pinned client's own item pipeline headlessly: the ClientItem codec,
 * ModelDiscovery, ModelBakery with real baked item models, ItemModelResolver
 * semantics and ItemStackRenderState layers. Sprites are real PNGs in a unit
 * atlas; the player skin cache and GPU are never touched.
 *
 * 1.21.7 has the older model system: BakedQuad, BlockModel, ItemTransform and
 * ItemModelGenerator live under renderer.block.model, there is no geometry /
 * cuboid / sprite split, SpecialModelRenderer.Unbaked is not generic, the
 * Unbaked item-model records carry no transformation, and the bakery takes no
 * MaterialSet — it resolves sprites through the SpriteGetter passed to
 * bakeModels. Further 1.21.7-only differences from 1.21.11: there is no
 * separate items atlas (one blocks atlas), SpecialModelRenderer renders via
 * render() rather than submit() with a SubmitNodeCollector, Unbaked.bake
 * takes the EntityModelSet, BakedQuad is a record whose vertices are read
 * raw (8 ints per vertex: position, color, uv, light), the sprite source
 * loader is SpriteSupplier, SpriteContents takes ResourceMetadata, and
 * there is no swapAnimationScale anywhere (ItemModelResolver,
 * ClientItem.Properties) and no ItemOwner (the resolver's owner is a
 * LivingEntity).
 */
public final class VanillaItemModelOracle {
    private static final Gson JSON = new GsonBuilder().serializeNulls().create();
    private static long bits(float value) { return Integer.toUnsignedLong(Float.floatToIntBits(value)); }
    static Map<String, Object> map(Object... values) {
        var result = new TreeMap<String, Object>();
        for (int i = 0; i < values.length; i += 2) result.put((String) values[i], values[i + 1]);
        return result;
    }

    static Path root;
    static HolderLookup.Provider registries;
    static RegistryOps<JsonElement> ops;
    static String frozenTime = "";
    // 1.21.7 has no items atlas: every sprite lives in the blocks atlas.
    static final ResourceLocation BLOCKS = TextureAtlas.LOCATION_BLOCKS, ITEMS = TextureAtlas.LOCATION_BLOCKS;

    static final class Sprite extends TextureAtlasSprite {
        Sprite(ResourceLocation atlas, SpriteContents contents) { super(atlas, contents, contents.width(), contents.height(), 0, 0); }
    }

    /** Sprites loaded on demand from the extracted jar. */
    static final class Regions extends HashMap<ResourceLocation, TextureAtlasSprite> {
        final boolean items;
        Regions(boolean items) { this.items = items; }
        @Override public TextureAtlasSprite get(Object key) {
            var id = (ResourceLocation) key;
            boolean itemAtlas = id.getPath().startsWith("item/") || id.getPath().startsWith("trims/");
            if (itemAtlas != items) return null;
            return computeIfAbsent(id, k -> load(k, items ? ITEMS : BLOCKS));
        }
    }
    // Loads a sprite's PNG for a paletted_permutations supplier: the supplier
    // resolves the template and palette resources itself and asks the loader
    // to read them.
    static final net.minecraft.client.renderer.texture.atlas.SpriteResourceLoader LOADER = (id, resource) -> {
        try {
            NativeImage image;
            try (var in = resource.open()) { image = NativeImage.read(in); }
            return new SpriteContents(id, new net.minecraft.client.resources.metadata.animation.FrameSize(image.getWidth(), image.getHeight()),
                image, net.minecraft.server.packs.resources.ResourceMetadata.EMPTY);
        } catch (IOException e) { throw new UncheckedIOException(e); }
    };

    static TextureAtlasSprite load(ResourceLocation id, ResourceLocation atlas) {
        try {
            var png = root.resolve("assets").resolve(id.getNamespace()).resolve("textures").resolve(id.getPath() + ".png");
            if (!Files.isRegularFile(png)) {
                // Armour trim textures come from the atlas's paletted_permutations source.
                var generated = generatedSprites(atlas).get(id);
                return generated == null ? null : new Sprite(atlas, generated.apply(LOADER));
            }
            NativeImage image;
            try (var in = Files.newInputStream(png)) { image = NativeImage.read(in); }
            var mcmeta = png.resolveSibling(png.getFileName() + ".mcmeta");
            Optional<AnimationMetadataSection> animation = Optional.empty();
            if (Files.isRegularFile(mcmeta)) {
                var json = JsonParser.parseString(Files.readString(mcmeta)).getAsJsonObject();
                if (json.has("animation"))
                    animation = Optional.of(AnimationMetadataSection.CODEC.parse(JsonOps.INSTANCE, json.get("animation")).getOrThrow());
            }
            var size = animation.map(a -> a.calculateFrameSize(image.getWidth(), image.getHeight()))
                .orElse(new net.minecraft.client.resources.metadata.animation.FrameSize(image.getWidth(), image.getHeight()));
            // 1.21.7's SpriteContents takes ResourceMetadata, not the
            // animation section: the frame size above is already
            // animation-adjusted, and the oracle never animates.
            return new Sprite(atlas, new SpriteContents(id, size, image, net.minecraft.server.packs.resources.ResourceMetadata.EMPTY));
        } catch (IOException e) { throw new UncheckedIOException(e); }
    }

    static final Map<ResourceLocation, Map<ResourceLocation, net.minecraft.client.renderer.texture.atlas.SpriteSource.SpriteSupplier>> GENERATED = new HashMap<>();
    /** The atlas's paletted_permutations sprites, run by vanilla's own source over the extracted resources. */
    static Map<ResourceLocation, net.minecraft.client.renderer.texture.atlas.SpriteSource.SpriteSupplier> generatedSprites(ResourceLocation atlas) {
        return GENERATED.computeIfAbsent(atlas, a -> {
            Map<ResourceLocation, net.minecraft.client.renderer.texture.atlas.SpriteSource.SpriteSupplier> out = new HashMap<>();
            var file = "/assets/" + a.getNamespace() + "/atlases/" + a.getPath().replaceFirst("^textures/atlas/", "").replaceFirst("\\.png$", "") + ".json";
            try (var in = VanillaItemModelOracle.class.getResourceAsStream(file)) {
                if (in == null) return out;
                var json = JsonParser.parseString(new String(in.readAllBytes(), java.nio.charset.StandardCharsets.UTF_8));
                var sources = net.minecraft.client.renderer.texture.atlas.SpriteSources.FILE_CODEC.parse(JsonOps.INSTANCE, json).getOrThrow();
                var output = new net.minecraft.client.renderer.texture.atlas.SpriteSource.Output() {
                    @Override public void add(ResourceLocation id, net.minecraft.client.renderer.texture.atlas.SpriteSource.SpriteSupplier sprite) { out.put(id, sprite); }
                    @Override public void removeAll(java.util.function.Predicate<ResourceLocation> predicate) { out.keySet().removeIf(predicate); }
                };
                for (var source : sources)
                    if (source instanceof net.minecraft.client.renderer.texture.atlas.sources.PalettedPermutations) source.run(FILES, output);
            } catch (IOException e) { throw new UncheckedIOException(e); }
            return out;
        });
    }
    /** The extracted resources as a single-pack resource manager. */
    static final net.minecraft.server.packs.resources.ResourceManager FILES = new net.minecraft.server.packs.resources.ResourceManager() {
        Path path(ResourceLocation id) { return root.resolve("assets").resolve(id.getNamespace()).resolve(id.getPath()); }
        @Override public Optional<net.minecraft.server.packs.resources.Resource> getResource(ResourceLocation id) {
            var file = path(id);
            return Files.isRegularFile(file) ? Optional.of(new net.minecraft.server.packs.resources.Resource(null, () -> Files.newInputStream(file))) : Optional.empty();
        }
        @Override public java.util.Set<String> getNamespaces() { return java.util.Set.of("minecraft"); }
        @Override public List<net.minecraft.server.packs.resources.Resource> getResourceStack(ResourceLocation id) { return getResource(id).stream().toList(); }
        @Override public Map<ResourceLocation, net.minecraft.server.packs.resources.Resource> listResources(String directory, java.util.function.Predicate<ResourceLocation> filter) { throw new UnsupportedOperationException(); }
        @Override public Map<ResourceLocation, List<net.minecraft.server.packs.resources.Resource>> listResourceStacks(String directory, java.util.function.Predicate<ResourceLocation> filter) { throw new UnsupportedOperationException(); }
        @Override public java.util.stream.Stream<net.minecraft.server.packs.PackResources> listPacks() { return java.util.stream.Stream.empty(); }
    };

    // --- Tagging of special renderers so the layer carries its codec form. ---
    // 1.21.7's Unbaked is not generic and bakes against the EntityModelSet
    // (no BakingContext); the renderer renders via render(), not submit(),
    // and getExtents fills a Set<Vector3f>.
    static final class Tagged implements SpecialModelRenderer.Unbaked {
        final SpecialModelRenderer.Unbaked delegate;
        Tagged(SpecialModelRenderer.Unbaked delegate) { this.delegate = delegate; }
        public MapCodec<? extends SpecialModelRenderer.Unbaked> type() { return delegate.type(); }
        public SpecialModelRenderer<?> bake(EntityModelSet models) {
            var baked = delegate.bake(models);
            if (baked == null) return null;
            JsonElement config = SpecialModelRenderers.CODEC.encodeStart(JsonOps.INSTANCE, delegate).getOrThrow();
            return new TaggedRenderer(baked, config);
        }
    }
    static final class TaggedRenderer implements SpecialModelRenderer<Object> {
        final SpecialModelRenderer<?> delegate; final JsonElement config;
        TaggedRenderer(SpecialModelRenderer<?> delegate, JsonElement config) { this.delegate = delegate; this.config = config; }
        public void render(Object argument, ItemDisplayContext context, com.mojang.blaze3d.vertex.PoseStack poseStack, MultiBufferSource buffer, int light, int overlay, boolean foil) {}
        public void getExtents(Set<Vector3f> output) { delegate.getExtents(output); }
        public Object extractArgument(ItemStack stack) { return new Argument(argument(config.getAsJsonObject().get("type").getAsString(), stack)); }
    }
    record Argument(JsonElement json) {}
    static JsonElement encode(DataComponentType<?> type, ItemStack stack) {
        var value = stack.get((DataComponentType) type);
        if (value == null) return JsonNull.INSTANCE;
        @SuppressWarnings({"unchecked", "rawtypes"}) var result = ((Codec) type.codecOrThrow()).encodeStart(ops, value).getOrThrow();
        return (JsonElement) result;
    }
    static JsonElement argument(String type, ItemStack stack) {
        return switch (type) {
            case "minecraft:banner" -> encode(DataComponents.BANNER_PATTERNS, stack);
            case "minecraft:decorated_pot" -> encode(DataComponents.POT_DECORATIONS, stack);
            case "minecraft:player_head" -> encode(DataComponents.PROFILE, stack);
            case "minecraft:shield" -> {
                var o = new JsonObject();
                var patterns = encode(DataComponents.BANNER_PATTERNS, stack);
                if (!patterns.isJsonNull()) o.add("minecraft:banner_patterns", patterns);
                var color = encode(DataComponents.BASE_COLOR, stack);
                if (!color.isJsonNull()) o.add("minecraft:base_color", color);
                yield o;
            }
            default -> JsonNull.INSTANCE;
        };
    }
    /** Stands in for LocalTime so a case can script the formatted time. */
    record FrozenTime(Codec<String> valueCodec) implements SelectItemModelProperty<String> {
        public String get(ItemStack s, ClientLevel l, net.minecraft.world.entity.LivingEntity o, int seed, ItemDisplayContext d) { return frozenTime; }
        @SuppressWarnings({"unchecked", "rawtypes"})
        public SelectItemModelProperty.Type type() { return net.minecraft.client.renderer.item.properties.select.LocalTime.TYPE; }
    }
    @SuppressWarnings({"unchecked", "rawtypes"})
    static ItemModel.Unbaked tag(ItemModel.Unbaked u) {
        // 1.21.7's Unbaked records carry no transformation.
        if (u instanceof SpecialModelWrapper.Unbaked s)
            return new SpecialModelWrapper.Unbaked(s.base(), new Tagged(s.specialModel()));
        if (u instanceof CompositeModel.Unbaked c)
            return new CompositeModel.Unbaked(c.models().stream().map(VanillaItemModelOracle::tag).collect(Collectors.toList()));
        if (u instanceof ConditionalItemModel.Unbaked c)
            return new ConditionalItemModel.Unbaked(c.property(), tag(c.onTrue()), tag(c.onFalse()));
        if (u instanceof SelectItemModel.Unbaked s) {
            var sw = (SelectItemModel.UnbakedSwitch) s.unbakedSwitch();
            var cases = new ArrayList<SelectItemModel.SwitchCase>();
            for (var c : (List<SelectItemModel.SwitchCase>) sw.cases()) cases.add(new SelectItemModel.SwitchCase(c.values(), tag(c.model())));
            Object property = sw.property();
            if (property instanceof net.minecraft.client.renderer.item.properties.select.LocalTime) property = new FrozenTime(Codec.STRING);
            return new SelectItemModel.Unbaked(new SelectItemModel.UnbakedSwitch((SelectItemModelProperty) property, cases), s.fallback().map(VanillaItemModelOracle::tag));
        }
        if (u instanceof RangeSelectItemModel.Unbaked r) {
            var entries = new ArrayList<RangeSelectItemModel.Entry>();
            for (var e : r.entries()) entries.add(new RangeSelectItemModel.Entry(e.threshold(), tag(e.model())));
            return new RangeSelectItemModel.Unbaked(r.property(), r.scale(), entries, r.fallback().map(VanillaItemModelOracle::tag));
        }
        return u;
    }

    // --- Scripted owner ---
    static final class Actor extends ArmorStand {
        boolean using; int remaining; ItemStack stack = ItemStack.EMPTY; HumanoidArm arm = HumanoidArm.RIGHT; EntityType<?> kind = EntityType.PLAYER;
        Actor() { super(null, null); }
        @Override public boolean isUsingItem() { return using; }
        @Override public ItemStack getUseItem() { return using ? stack : ItemStack.EMPTY; }
        @Override public int getUseItemRemainingTicks() { return remaining; }
        @Override public HumanoidArm getMainArm() { return arm; }
        @Override public EntityType<?> getType() { return kind; }
    }
    static Object unsafe; static Method allocate;
    @SuppressWarnings("unchecked") static <T> T make(Class<T> type) throws Exception { return (T) allocate.invoke(unsafe, type); }

    static final class Resolver extends ItemModelResolver {
        Map<ResourceLocation, ItemModel> models; Map<ResourceLocation, ClientItem.Properties> properties; ItemModel missing;
        Resolver() { super(null); }
        // 1.21.7 has no ItemOwner: the resolver's owner is a LivingEntity.
        @Override public void appendItemLayers(ItemStackRenderState output, ItemStack item, ItemDisplayContext context, Level level, LivingEntity owner, int seed) {
            ResourceLocation id = item.get(DataComponents.ITEM_MODEL);
            if (id != null) {
                output.setOversizedInGui(properties.getOrDefault(id, ClientItem.Properties.DEFAULT).oversizedInGui());
                models.getOrDefault(id, missing).update(output, item, this, context, level instanceof ClientLevel c ? c : null, owner, seed);
            }
        }
        // ModelManager.getItemProperties, from the same baked properties.
        // 1.21.7 has no swapAnimationScale (neither on ItemModelResolver
        // nor on ClientItem.Properties): the swap output below drops it.
        @Override public boolean shouldPlaySwapAnimation(ItemStack stack) {
            ResourceLocation id = stack.get(DataComponents.ITEM_MODEL);
            return id == null || properties.getOrDefault(id, ClientItem.Properties.DEFAULT).handAnimationOnSwap();
        }
    }

    static Object field(Object target, Class<?> type, String name) throws Exception {
        var f = type.getDeclaredField(name); f.setAccessible(true); return f.get(target);
    }
    static Object quad(BakedQuad q) {
        // 1.21.7's BakedQuad is a record with no position()/packedUV(): the
        // vertices are read raw — 8 ints per vertex (FaceBakery bakes a
        // 32-int array: position at +0..+2, color at +3, uv at +4..+5).
        // The ints already are the float bits, so the unsigned-long values
        // match the 26.x form's bits()/packed split exactly.
        var v = q.vertices();
        var positions = new long[4][3]; var uvs = new long[4][2];
        for (int i = 0; i < 4; i++) {
            int base = i * 8;
            positions[i] = new long[]{Integer.toUnsignedLong(v[base]), Integer.toUnsignedLong(v[base + 1]), Integer.toUnsignedLong(v[base + 2])};
            uvs[i] = new long[]{Integer.toUnsignedLong(v[base + 4]), Integer.toUnsignedLong(v[base + 5])};
        }
        return map("positions", positions, "uv", uvs, "facing", q.direction().getName(), "tint", q.tintIndex(),
            "sprite", q.sprite().contents().name().toString(),
            "shade", q.shade(), "light", q.lightEmission());
    }
    static Object matrix(Matrix4f m) {
        var rows = new long[16];
        for (int r = 0; r < 4; r++) for (int c = 0; c < 4; c++) rows[r * 4 + c] = bits(m.getRowColumn(r, c));
        return rows;
    }
    static Object layer(ItemStackRenderState state, Object layer, ItemDisplayContext context) throws Exception {
        var type = layer.getClass();
        var out = new TreeMap<String, Object>();
        var special = (SpecialModelRenderer<?>) field(layer, type, "specialRenderer");
        var tints = (int[]) field(layer, type, "tintLayers");
        out.put("tints", tints == null ? new int[0] : tints);
        out.put("foil", ((ItemStackRenderState.FoilType) field(layer, type, "foilType")).name().toLowerCase(Locale.ROOT));
        var pose = new com.mojang.blaze3d.vertex.PoseStack.Pose();
        var transform = (ItemTransform) field(layer, type, "transform");
        if (transform != null) transform.apply(false, pose);
        out.put("matrix", matrix(pose.pose()));
        out.put("uses_block_light", field(layer, type, "usesBlockLight"));
        var particle = (net.minecraft.client.renderer.texture.TextureAtlasSprite) field(layer, type, "particleIcon");
        out.put("particle", particle == null ? null : particle.contents().name().toString());
        out.put("has_transform", transform != null && transform != ItemTransform.NO_TRANSFORM);
        if (special instanceof TaggedRenderer tagged) {
            out.put("kind", "special");
            out.put("special", tagged.config);
            var argument = field(layer, type, "argumentForSpecialRendering");
            out.put("argument", argument instanceof Argument a ? a.json() : JsonNull.INSTANCE);
        } else {
            var quads = (java.util.List<BakedQuad>) field(layer, type, "quads");
            out.put("quads", quads.stream().map(VanillaItemModelOracle::quad).toList());
            out.put("kind", "model");
        }
        return out;
    }

    static Map<String, JsonObject> prototypes = new HashMap<>();

    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        // Item tags (compasses get the special foil) come from the vanilla data pack.
        var repo = net.minecraft.server.packs.repository.ServerPacksSource.createVanillaTrustedRepository();
        repo.reload();
        repo.setSelected(List.of("vanilla"));
        try (var data = new net.minecraft.server.packs.resources.MultiPackResourceManager(net.minecraft.server.packs.PackType.SERVER_DATA, repo.openAllSelected())) {
            var layers = net.minecraft.server.RegistryLayer.createRegistryAccess();
            net.minecraft.tags.TagLoader.loadTagsForExistingRegistries(data, layers.getLayer(net.minecraft.server.RegistryLayer.STATIC))
                .forEach(net.minecraft.core.Registry.PendingTags::apply);
        }
        ItemModels.bootstrap();
        SpecialModelRenderers.bootstrap();
        net.minecraft.client.color.item.ItemTintSources.bootstrap();
        net.minecraft.client.renderer.item.properties.select.SelectItemModelProperties.bootstrap();
        net.minecraft.client.renderer.item.properties.conditional.ConditionalItemModelProperties.bootstrap();
        net.minecraft.client.renderer.item.properties.numeric.RangeSelectItemModelProperties.bootstrap();
        net.minecraft.client.renderer.texture.atlas.SpriteSources.bootstrap();
        var u = Class.forName("sun.misc.Unsafe");
        var uf = u.getDeclaredField("theUnsafe"); uf.setAccessible(true); unsafe = uf.get(null);
        allocate = u.getMethod("allocateInstance", Class.class);
        root = Path.of(args[0]);
        registries = VanillaRegistries.createLookup();
        ops = RegistryOps.create(JsonOps.INSTANCE, registries);
        // Default components come from the prepared export of the vanilla reports.
        // 1.21.7 has no Holder.Reference.bindComponents: the components live on
        // the Item itself, so they are set through its private final field.
        var defaults = JsonParser.parseString(Files.readString(Path.of(args[3]))).getAsJsonObject();
        var componentsField = Item.class.getDeclaredField("components"); componentsField.setAccessible(true);
        for (var item : BuiltInRegistries.ITEM) {
            var json = defaults.get(BuiltInRegistries.ITEM.getKey(item).toString());
            var map = json == null ? net.minecraft.core.component.DataComponentMap.EMPTY
                : net.minecraft.core.component.DataComponentMap.CODEC.parse(ops, json).getOrThrow();
            componentsField.set(item, map);
        }

        // Grass colour map for the grass tint.
        try (var in = Files.newInputStream(root.resolve("assets/minecraft/textures/colormap/grass.png")); var image = NativeImage.read(in)) {
            GrassColor.init(image.getPixels());
        }

        // Every model file.
        var models = new HashMap<ResourceLocation, UnbakedModel>();
        try (Stream<Path> files = Files.walk(root.resolve("assets/minecraft/models"))) {
            for (var file : (Iterable<Path>) files.filter(p -> p.toString().endsWith(".json"))::iterator) {
                var relative = root.resolve("assets/minecraft/models").relativize(file).toString().replace('\\', '/');
                try (var reader = Files.newBufferedReader(file)) {
                    models.put(ResourceLocation.withDefaultNamespace(relative.substring(0, relative.length() - 5)), BlockModel.fromStream(reader));
                } catch (RuntimeException ignored) {}
            }
        }
        // Every item definition.
        var output = Files.newBufferedWriter(Path.of(args[2]));
        output.write(JSON.toJson(map("row", "components", "names",
            BuiltInRegistries.DATA_COMPONENT_TYPE.keySet().stream().map(ResourceLocation::toString).sorted().toList()))); output.newLine();
        var items = new TreeMap<ResourceLocation, ClientItem>();
        try (Stream<Path> files = Files.list(root.resolve("assets/minecraft/items"))) {
            for (var file : (Iterable<Path>) files.sorted()::iterator) {
                var name = file.getFileName().toString();
                var id = ResourceLocation.withDefaultNamespace(name.substring(0, name.length() - 5));
                String text = Files.readString(file);
                boolean ok = decode(id, text, items);
                output.write(JSON.toJson(map("row", "decode", "name", name, "ok", ok))); output.newLine();
            }
        }
        // Synthetic definitions from the harness.
        try (var lines = Files.lines(Path.of(args[1] + ".definitions"))) {
            for (var line : (Iterable<String>) lines::iterator) {
                var row = JsonParser.parseString(line).getAsJsonObject();
                var id = ResourceLocation.parse(row.get("name").getAsString());
                boolean ok = decode(id, row.get("json").getAsString(), items);
                output.write(JSON.toJson(map("row", "decode", "name", row.get("name").getAsString(), "ok", ok))); output.newLine();
            }
        }

        var missingModel = MissingBlockModel.missingModel();
        var discovery = new ModelDiscovery(models, missingModel);
        discovery.addSpecialModel(ItemModelGenerator.GENERATED_ITEM_MODEL_ID, new ItemModelGenerator());
        items.values().forEach(info -> discovery.addRoot(info.model()));
        var resolved = discovery.resolve();
        var missingSprite = new Sprite(BLOCKS, MissingTextureAtlasSprite.create());
        var blockRegions = new Regions(false);
        var itemRegions = new Regions(true);
        // 1.21.7 has no MaterialBaker or MaterialSet: the bakery resolves
        // sprites through the SpriteGetter passed to bakeModels, item atlas
        // first with the block atlas as fallback, then the missing sprite.
        // (Both regions load from the one blocks atlas; see ITEMS above.)
        SpriteGetter sprites = new SpriteGetter() {
            @Override public TextureAtlasSprite get(Material material, ModelDebugName debugName) {
                var sprite = itemRegions.get(material.texture());
                if (sprite == null) sprite = blockRegions.get(material.texture());
                return sprite != null ? sprite : missingSprite;
            }
            @Override public TextureAtlasSprite reportMissingReference(String reference, ModelDebugName debugName) { return missingSprite; }
        };
        var bakery = new ModelBakery(EntityModelSet.vanilla(), Map.of(), tagged(items), resolved, discovery.missingModel());
        var baked = bakery.bakeModels(sprites, Runnable::run).join();
        var resolver = new Resolver();
        resolver.models = baked.itemStackModels(); resolver.properties = baked.itemProperties(); resolver.missing = baked.missingModels().item();
        for (var id : items.keySet()) if (!resolver.models.containsKey(id)) {
            output.write(JSON.toJson(map("row", "unbaked", "name", id.toString()))); output.newLine();
        }
        var state = new ItemStackRenderState();

        try (var lines = Files.lines(Path.of(args[1]))) {
            for (var line : (Iterable<String>) lines::iterator) {
                var row = JsonParser.parseString(line).getAsJsonObject();
                var result = new TreeMap<String, Object>();
                result.put("row", "case"); result.put("id", row.get("id").getAsString());
                try {
                    var stackJson = new JsonObject();
                    stackJson.addProperty("id", row.get("item").getAsString());
                    stackJson.addProperty("count", row.get("count").getAsInt());
                    if (row.has("components")) stackJson.add("components", row.get("components"));
                    var stack = ItemStack.CODEC.parse(ops, stackJson).getOrThrow();
                    if (row.has("bundle_selected") && row.get("bundle_selected").getAsInt() >= 0) {
                        var contents = stack.get(DataComponents.BUNDLE_CONTENTS);
                        // 1.21.7 has no BundleContents.Mutable: the selected index
                        // is set only by the package-private (items, weight, selected)
                        // constructor, with the weight carried over unchanged.
                        var ctor = BundleContents.class.getDeclaredConstructor(java.util.List.class, org.apache.commons.lang3.math.Fraction.class, int.class);
                        ctor.setAccessible(true);
                        var itemsField = BundleContents.class.getDeclaredField("items"); itemsField.setAccessible(true);
                        var weightField = BundleContents.class.getDeclaredField("weight"); weightField.setAccessible(true);
                        stack.set(DataComponents.BUNDLE_CONTENTS, ctor.newInstance(itemsField.get(contents), weightField.get(contents), row.get("bundle_selected").getAsInt()));
                    }
                    var context = ItemDisplayContext.CODEC.parse(JsonOps.INSTANCE, row.get("context")).getOrThrow();
                    frozenTime = row.has("local_time") ? row.get("local_time").getAsString() : "";
                    Actor actor = null;
                    if (row.has("owner") && !row.get("owner").isJsonNull()) {
                        var o = row.getAsJsonObject("owner");
                        actor = make(Actor.class);
                        actor.using = o.get("using").getAsBoolean();
                        actor.remaining = o.get("remaining").getAsInt();
                        actor.stack = stack;
                        actor.arm = o.get("main_arm").getAsString().equals("left") ? HumanoidArm.LEFT : HumanoidArm.RIGHT;
                        actor.kind = BuiltInRegistries.ENTITY_TYPE.getValue(ResourceLocation.parse(o.get("entity_type").getAsString()));
                        var derived = new TreeMap<String, Object>();
                        derived.put("use_duration", stack.getUseDuration(actor));
                        try { derived.put("charge_duration", CrossbowItem.getChargeDuration(stack, actor)); } catch (RuntimeException e) { derived.put("charge_duration", 0); }
                        result.put("derived", derived);
                    }
                    Level level = null;
                    if (row.has("dimension") && !row.get("dimension").isJsonNull()) {
                        var dim = net.minecraft.resources.ResourceKey.create(net.minecraft.core.registries.Registries.DIMENSION, ResourceLocation.parse(row.get("dimension").getAsString()));
                        var clientLevel = make(ClientLevel.class);
                        var f = Level.class.getDeclaredField("dimension"); f.setAccessible(true);
                        var offset = (long) u.getMethod("objectFieldOffset", Field.class).invoke(unsafe, f);
                        u.getMethod("putObject", Object.class, long.class, Object.class).invoke(unsafe, clientLevel, offset, dim);
                        level = clientLevel;
                    }
                    resolver.updateForTopItem(state, stack, context, level, actor, row.has("seed") ? row.get("seed").getAsInt() : 0);
                    var layersField = ItemStackRenderState.class.getDeclaredField("layers"); layersField.setAccessible(true);
                    var countField = ItemStackRenderState.class.getDeclaredField("activeLayerCount"); countField.setAccessible(true);
                    var layers = (Object[]) layersField.get(state); int count = countField.getInt(state);
                    var list = new ArrayList<Object>();
                    for (int i = 0; i < count; i++) list.add(layer(state, layers[i], context));
                    result.put("layers", list);
                    result.put("oversized", state.isOversizedInGui());
                    // 1.21.7 has no swap scale: only the play flag is reported.
                    result.put("swap", map("play", resolver.shouldPlaySwapAnimation(stack)));
                } catch (Throwable e) {
                    Throwable cause = e; while (cause.getCause() != null) cause = cause.getCause();
                    result.put("error", cause.toString());
                }
                output.write(JSON.toJson(result)); output.newLine();
            }
        }
        output.close();
    }

    static Map<ResourceLocation, ClientItem> tagged(Map<ResourceLocation, ClientItem> items) {
        var out = new TreeMap<ResourceLocation, ClientItem>();
        items.forEach((id, item) -> out.put(id, new ClientItem(tag(item.model()), item.properties(), item.registrySwapper())));
        return out;
    }
    static boolean decode(ResourceLocation id, String text, Map<ResourceLocation, ClientItem> items) {
        try {
            var parsed = ClientItem.CODEC.parse(ops, StrictJsonParser.parse(new StringReader(text))).result();
            parsed.ifPresent(item -> items.put(id, item));
            return parsed.isPresent();
        } catch (Exception e) { return false; }
    }
}
