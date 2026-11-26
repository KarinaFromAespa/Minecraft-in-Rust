package dev.horizonlab.parity;

import com.google.gson.*;
import com.mojang.datafixers.DataFixUtils;
import com.mojang.datafixers.schemas.Schema;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Dynamic;
import com.mojang.serialization.JsonOps;
import it.unimi.dsi.fastutil.ints.Int2ObjectMap;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import java.io.ByteArrayOutputStream;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.datafix.DataFixers;

/**
 * Original exporter of the lookup tables of the 1.13-era data fixers (DataVersion 99 to 1631), read
 * from the player's own server jar: the id mappings horizon_world::datafix needs to upgrade saves
 * made before 1.13 (datafix-tables.json, loaded by horizon-world's datafix/tables.rs).
 *
 * <p>Maps and sets are written sorted by key: every Horizon use is a lookup by key or a membership
 * test. {@code BlockStateData} is the exception, its order is the table's meaning (the first state
 * registered for a block is its default, the first legacy name registered wins), so it is recorded
 * while vanilla's own static initializer runs: one line per {@code register} call, in call order.
 */
public final class VanillaDataFixTablesExporter {
    private static final String FIXES = "net.minecraft.util.datafix.fixes.";
    private static final String BLOCK_STATE_DATA = FIXES + "BlockStateData";
    private static final String REGISTER = "(ILcom/mojang/serialization/Dynamic;[Lcom/mojang/serialization/Dynamic;)V";

    /** The {@code register} calls seen by {@link #record}, in order: the new state, then the legacy ones. */
    private static final List<Dynamic<?>[]> CALLS = new ArrayList<>();
    private static final List<Integer> IDS = new ArrayList<>();

    /** Called first thing in vanilla's {@code BlockStateData.register} (see {@link #recordedBlockStateData}). */
    public static void record(int id, Dynamic<?> state, Dynamic<?>[] legacy) {
        var all = new Dynamic<?>[legacy.length + 1];
        all[0] = state;
        System.arraycopy(legacy, 0, all, 1, legacy.length);
        IDS.add(id);
        CALLS.add(all);
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var out = new JsonObject();
        pairs(out, "ITEM_FLATTENING", stringMap(FIXES + "ItemStackTheFlatteningFix", "MAP"));
        strings(out, "ITEM_DAMAGE_IDS", stringSet(FIXES + "ItemStackTheFlatteningFix", "DAMAGE_IDS"));
        var blockIds = new JsonArray();
        Map<String, Integer> blocks = cast(field(FIXES + "EntityBlockStateFix", "MAP"));
        for (var e : new TreeMap<>(nonEmpty(blocks, "MAP")).entrySet())
            blockIds.add(pair(new JsonPrimitive(e.getKey()), new JsonPrimitive(e.getValue())));
        out.add("BLOCK_IDS", blockIds);
        intPairs(out, "ITEM_IDS", intMap(FIXES + "ItemIdFix", "ITEM_NAMES"));
        strings(out, "SPECIAL_OBJECTIVE_CRITERIA", stringSet(FIXES + "StatsCounterFix", "SPECIAL_OBJECTIVE_CRITERIA"));
        strings(out, "STATS_SKIP", stringSet(FIXES + "StatsCounterFix", "SKIP"));
        pairs(out, "STATS_CUSTOM", stringMap(FIXES + "StatsCounterFix", "CUSTOM_MAP"));
        pairs(out, "STATS_ITEM_KEYS", stringMap(FIXES + "StatsCounterFix", "ITEM_KEYS"));
        pairs(out, "STATS_ENTITY_KEYS", stringMap(FIXES + "StatsCounterFix", "ENTITY_KEYS"));
        pairs(out, "STATS_ENTITIES", stringMap(FIXES + "StatsCounterFix", "ENTITIES"));
        pairs(out, "RECIPES", stringMap(FIXES + "RecipesFix", "RECIPES"));
        pairs(out, "RECIPES_RENAMENING", stringMap(FIXES + "RecipesRenameningFix", "RECIPES"));
        pairs(out, "ADVANCEMENTS", stringMap(FIXES + "AdvancementsFix", "RENAMES"));
        pairs(out, "GENERATOR_BIOMES", stringMap(FIXES + "LevelDataGeneratorOptionsFix", "MAP"));
        pairs(out, "SPAWN_EGGS", stringMap(FIXES + "ItemStackSpawnEggFix", "MAP"));
        intPairs(out, "ENCHANTMENTS", intMap(FIXES + "ItemStackEnchantmentNamesFix", "MAP"));
        pairs(out, "CORAL", stringMap(FIXES + "RenamedCoralFix", "RENAMED_IDS"));
        pairs(out, "CORAL_FANS", stringMap(FIXES + "RenamedCoralFansFix", "RENAMED_IDS"));
        pairs(out, "THE_RENAMENING_ENTITIES", stringMap(FIXES + "EntityTheRenameningFix", "RENAMED_IDS"));
        pairs(out, "THE_RENAMENING_BLOCKS", stringMap(FIXES + "EntityTheRenameningFix", "RENAMED_BLOCKS"));
        pairs(out, "COD_SALMON", stringMap(FIXES + "EntityCodSalmonFix", "RENAMED_IDS"));
        pairs(out, "COD_SALMON_EGGS", stringMap(FIXES + "EntityCodSalmonFix", "RENAMED_EGG_IDS"));
        var fixer = DataFixers.getDataFixer();
        var v1460 = schema(fixer.getSchema(DataFixUtils.makeKey(1460)), "V1460");
        var v99 = schema(fixer.getSchema(DataFixUtils.makeKey(99)), "V99");
        strings(out, "ENTITIES_V1460", new TreeSet<>(v1460.registerEntities(v1460).keySet()));
        strings(out, "ENTITIES_V99", new TreeSet<>(v99.registerEntities(v99).keySet()));
        strings(out, "BLOCK_ENTITIES_V99", new TreeSet<>(v99.registerBlockEntities(v99).keySet()));
        indexed(out, "SPAWN_EGG_ENTITIES", (String[]) field(FIXES + "ItemSpawnEggFix", "ID_TO_ENTITY"));
        pairs(out, "ENTITY_IDS", stringMap(FIXES + "EntityIdFix", "ID_MAP"));
        pairs(out, "BLOCK_ENTITY_IDS", stringMap(FIXES + "BlockEntityIdFix", "ID_MAP"));
        pairs(out, "ITEM_TO_BLOCK_ENTITY_V99", stringMap("net.minecraft.util.datafix.schemas.V99", "ITEM_TO_BLOCKENTITY"));
        indexed(out, "ITEM_POTIONS", (String[]) field(FIXES + "ItemPotionFix", "POTIONS"));
        var renames = new JsonArray();
        Map<String, Pair<String, Map<String, String>>> structures = cast(field(FIXES + "ChunkStructuresTemplateRenameFix", "RENAMES"));
        for (var e : new TreeMap<>(structures).entrySet()) {
            var templates = new JsonArray();
            for (var t : new TreeMap<>(e.getValue().getSecond()).entrySet())
                templates.add(pair(new JsonPrimitive(t.getKey()), new JsonPrimitive(t.getValue())));
            var row = pair(new JsonPrimitive(e.getKey()), new JsonPrimitive(e.getValue().getFirst()));
            row.add(templates);
            renames.add(row);
        }
        out.add("STRUCTURE_TEMPLATE_RENAMES", renames);
        var lines = new JsonArray();
        for (String line : blockStateLines()) lines.add(line);
        out.add("BLOCK_STATE_DATA", lines);
        Files.writeString(Path.of(args[0]), new GsonBuilder().disableHtmlEscaping().create().toJson(out), StandardCharsets.UTF_8);
    }

    // ------------------------------------------------------------------ BlockStateData

    /**
     * The {@code register} calls of {@code BlockStateData}'s static initializer, one tab-separated line
     * each: {@code id, name, props} of the new state, then {@code name, props} of every legacy state;
     * props are {@code key=value} joined by {@code ;}, sorted by key, empty when there are none.
     */
    private static List<String> blockStateLines() throws Exception {
        CALLS.clear();
        IDS.clear();
        var type = recordedBlockStateData();
        if (CALLS.isEmpty()) throw new IllegalStateException("BlockStateData registered nothing");
        checkReplay(type);
        var lines = new ArrayList<String>(CALLS.size());
        for (int i = 0; i < CALLS.size(); i++) {
            var line = new StringBuilder().append(IDS.get(i));
            for (var state : CALLS.get(i)) {
                var json = state.convert(JsonOps.INSTANCE).getValue().getAsJsonObject();
                var keys = new TreeSet<>(json.keySet());
                if (!keys.contains("Name") || !Set.of("Name", "Properties").containsAll(keys))
                    throw new IllegalStateException("unexpected state " + json);
                line.append('\t').append(json.get("Name").getAsString()).append('\t');
                if (json.has("Properties")) {
                    var props = new TreeMap<String, String>();
                    for (var p : json.getAsJsonObject("Properties").entrySet()) props.put(p.getKey(), p.getValue().getAsString());
                    var joined = new StringJoiner(";");
                    props.forEach((k, v) -> joined.add(k + "=" + v));
                    line.append(joined);
                }
            }
            lines.add(line.toString());
        }
        return lines;
    }

    /**
     * Initialises a private copy of {@code BlockStateData} whose {@code register} first calls
     * {@link #record}; the rest of the class is the jar's, unchanged.
     */
    private static Class<?> recordedBlockStateData() throws Exception {
        var parent = VanillaDataFixTablesExporter.class.getClassLoader();
        byte[] original;
        try (var in = parent.getResourceAsStream(BLOCK_STATE_DATA.replace('.', '/') + ".class")) {
            if (in == null) throw new IllegalStateException(BLOCK_STATE_DATA + " not on the classpath");
            original = in.readAllBytes();
        }
        byte[] patched = hookRegisterCalls(original);
        var loader = new ClassLoader(parent) {
            @Override
            protected Class<?> loadClass(String name, boolean resolve) throws ClassNotFoundException {
                if (!name.equals(BLOCK_STATE_DATA)) return super.loadClass(name, resolve);
                synchronized (getClassLoadingLock(name)) {
                    var loaded = findLoadedClass(name);
                    return loaded != null ? loaded : defineClass(name, patched, 0, patched.length);
                }
            }
        };
        return Class.forName(BLOCK_STATE_DATA, true, loader);
    }

    // ------------------------------------------------------------------ class-file patch

    /**
     * Inserts the {@link #record} call at the start of {@code BlockStateData.register}. Newer JDKs do
     * this with the Class-File API (java.lang.classfile, JDK 22+); 1.21.11's JDK 21 does not have it,
     * so the same edit is done here directly on the class bytes: the record() methodref is appended
     * to the constant pool and the six hook bytes (iload_0, aload_1, aload_2, invokestatic) are
     * prepended to the method's code, with every pc-bearing table shifted by the insertion size.
     */
    private static byte[] hookRegisterCalls(byte[] original) throws Exception {
        var in = new Reader(original);
        in.u4(); // magic
        in.u2();
        in.u2(); // minor, major
        int cpCount = in.u2();
        var utf8 = new HashMap<Integer, String>();
        for (int i = 1; i < cpCount; i++) {
            int tag = in.u1();
            switch (tag) {
                case 1 -> {
                    int len = in.u2();
                    utf8.put(i, new String(in.bytes(len), StandardCharsets.UTF_8));
                }
                case 3, 4 -> in.skip(4);
                case 5, 6 -> {
                    in.skip(8);
                    i++;
                }
                case 7, 8, 16, 19, 20 -> in.skip(2);
                case 9, 10, 11, 12, 17, 18 -> in.skip(4);
                case 15 -> in.skip(3);
                default -> throw new IllegalStateException("constant pool tag " + tag);
            }
        }
        int cpEnd = in.pos;
        in.skip(6); // access flags, this class, super class
        int interfaces = in.u2();
        in.skip(2 * interfaces);
        skipMembers(in); // fields
        int methods = in.u2();
        int codeAttrStart = -1;
        int codeAttrLen = 0;
        int found = 0;
        for (int m = 0; m < methods; m++) {
            in.skip(2); // access flags
            int nameIdx = in.u2();
            int descIdx = in.u2();
            boolean target = "register".equals(utf8.get(nameIdx)) && REGISTER.equals(utf8.get(descIdx));
            int attrs = in.u2();
            for (int a = 0; a < attrs; a++) {
                int attrName = in.u2();
                int attrLen = (int) in.u4();
                if (target && "Code".equals(utf8.get(attrName))) {
                    found++;
                    codeAttrStart = in.pos;
                    codeAttrLen = attrLen;
                }
                in.skip(attrLen);
            }
        }
        if (found != 1) throw new IllegalStateException("BlockStateData.register" + REGISTER + " not found");

        var code = new Reader(original, codeAttrStart);
        int maxStack = code.u2();
        int maxLocals = code.u2();
        int codeLen = (int) code.u4();
        int codeStart = code.pos;
        code.skip(codeLen);
        int excCount = code.u2();
        int excStart = code.pos;
        code.skip(excCount * 8);
        int codeAttrs = code.u2();
        var subs = new ArrayList<int[]>(); // {name index, content start, length}
        for (int a = 0; a < codeAttrs; a++) {
            int nameIdx = code.u2();
            int len = (int) code.u4();
            subs.add(new int[]{nameIdx, code.pos, len});
            code.skip(len);
        }

        int hookLen = 6; // iload_0, aload_1, aload_2, invokestatic (opcode + u2 index)
        int recordRef = cpCount + 5; // the appended Methodref, after Utf8/Class/Utf8/Utf8/NameAndType
        var pool = new ByteArrayOutputStream();
        writeUtf8(pool, VanillaDataFixTablesExporter.class.getName().replace('.', '/'));
        pool.write(7);
        writeU2(pool, cpCount); // Class -> the class-name Utf8
        writeUtf8(pool, "record");
        writeUtf8(pool, REGISTER);
        pool.write(12); // NameAndType: "record", REGISTER
        writeU2(pool, cpCount + 2);
        writeU2(pool, cpCount + 3);
        pool.write(10); // Methodref: the Class, the NameAndType
        writeU2(pool, cpCount + 1);
        writeU2(pool, cpCount + 4);

        var patchedSubs = new ArrayList<byte[]>();
        int subGrowth = 0;
        for (int[] sub : subs) {
            byte[] content = patchCodeAttribute(utf8.get(sub[0]), original, sub[1], sub[2], hookLen);
            subGrowth += content.length - sub[2];
            patchedSubs.add(content);
        }

        var out = new ByteArrayOutputStream(original.length + pool.size() + hookLen + subGrowth);
        byte[] head = Arrays.copyOfRange(original, 0, cpEnd);
        head[8] = (byte) ((cpCount + 6) >>> 8);
        head[9] = (byte) (cpCount + 6);
        out.write(head);
        out.write(pool.toByteArray());
        out.write(original, cpEnd, codeAttrStart - cpEnd); // includes the Code attribute header
        writeU2(out, Math.max(maxStack, 3));
        writeU2(out, maxLocals);
        writeU4(out, codeLen + hookLen);
        out.write(0x1A); // iload_0
        out.write(0x2B); // aload_1
        out.write(0x2C); // aload_2
        out.write(0xB8); // invokestatic
        writeU2(out, recordRef);
        out.write(original, codeStart, codeLen);
        writeU2(out, excCount);
        for (int i = 0; i < excCount; i++) {
            int base = excStart + i * 8;
            writeU2(out, u2At(original, base) + hookLen);
            writeU2(out, u2At(original, base + 2) + hookLen);
            writeU2(out, u2At(original, base + 4) + hookLen);
            writeU2(out, u2At(original, base + 6));
        }
        writeU2(out, codeAttrs);
        for (int s = 0; s < subs.size(); s++) {
            writeU2(out, subs.get(s)[0]);
            byte[] content = patchedSubs.get(s);
            writeU4(out, content.length);
            out.write(content);
        }
        int after = codeAttrStart + codeAttrLen;
        out.write(original, after, original.length - after);
        byte[] result = out.toByteArray();
        // The Code attribute_length field sits four bytes before its content; only the pool grew before it.
        int lenPos = codeAttrStart - 4 + pool.size();
        int newLen = codeAttrLen + hookLen + subGrowth;
        result[lenPos] = (byte) (newLen >>> 24);
        result[lenPos + 1] = (byte) (newLen >>> 16);
        result[lenPos + 2] = (byte) (newLen >>> 8);
        result[lenPos + 3] = (byte) newLen;
        return result;
    }

    private static void skipMembers(Reader in) {
        int count = in.u2();
        for (int i = 0; i < count; i++) {
            in.skip(6);
            int attrs = in.u2();
            for (int a = 0; a < attrs; a++) {
                in.skip(2);
                in.skip((int) in.u4());
            }
        }
    }

    /** A Code sub-attribute with its pc references shifted past the inserted hook bytes. */
    private static byte[] patchCodeAttribute(String name, byte[] bytes, int start, int len, int hook) {
        byte[] content = Arrays.copyOfRange(bytes, start, start + len);
        if (name == null) return content;
        switch (name) {
            case "LineNumberTable" -> {
                int entries = u2At(content, 0);
                for (int i = 0; i < entries; i++) putU2(content, 2 + i * 4, u2At(content, 2 + i * 4) + hook);
            }
            case "LocalVariableTable", "LocalVariableTypeTable" -> {
                int entries = u2At(content, 0);
                for (int i = 0; i < entries; i++) putU2(content, 2 + i * 10, u2At(content, 2 + i * 10) + hook);
            }
            case "StackMapTable" -> {
                return shiftStackMapTable(content, hook);
            }
            default -> {
                return content;
            }
        }
        return content;
    }

    /**
     * Every frame's absolute pc moves by {@code hook}; frame offsets are deltas, so only the first
     * frame's delta changes (re-encoded in the extended form when it outgrows the compact tags).
     */
    private static byte[] shiftStackMapTable(byte[] table, int hook) {
        var in = new Reader(table);
        int entries = in.u2();
        if (entries == 0) return table;
        int tag = in.u1();
        var out = new ByteArrayOutputStream(table.length + 2);
        writeU2(out, entries);
        if (tag <= 63) { // same_frame
            int delta = tag + hook;
            if (delta <= 63) {
                out.write(delta);
            } else {
                out.write(251); // same_frame_extended
                writeU2(out, delta);
            }
        } else if (tag <= 127) { // same_locals_1_stack_item
            int delta = tag - 64 + hook;
            int vStart = in.pos;
            skipVerificationType(in);
            int vEnd = in.pos;
            if (delta <= 63) {
                out.write(64 + delta);
            } else {
                out.write(247); // same_locals_1_stack_item_extended
                writeU2(out, delta);
            }
            out.write(table, vStart, vEnd - vStart);
        } else { // every remaining frame form carries an explicit u2 offset_delta
            int delta = in.u2() + hook;
            out.write(tag);
            writeU2(out, delta);
        }
        out.write(table, in.pos, table.length - in.pos);
        return out.toByteArray();
    }

    private static void skipVerificationType(Reader in) {
        int tag = in.u1();
        if (tag == 7 || tag == 8) in.skip(2); // Object, Uninitialized
    }

    private static int u2At(byte[] bytes, int pos) {
        return ((bytes[pos] & 0xFF) << 8) | (bytes[pos + 1] & 0xFF);
    }

    private static void putU2(byte[] bytes, int pos, int value) {
        bytes[pos] = (byte) (value >>> 8);
        bytes[pos + 1] = (byte) value;
    }

    private static void writeU2(ByteArrayOutputStream out, int value) {
        out.write(value >>> 8);
        out.write(value);
    }

    private static void writeU4(ByteArrayOutputStream out, int value) {
        out.write(value >>> 24);
        out.write(value >>> 16);
        out.write(value >>> 8);
        out.write(value);
    }

    private static void writeUtf8(ByteArrayOutputStream out, String value) throws Exception {
        byte[] bytes = value.getBytes(StandardCharsets.UTF_8);
        out.write(1);
        writeU2(out, bytes.length);
        out.write(bytes);
    }

    private static final class Reader {
        final byte[] bytes;
        int pos;

        Reader(byte[] bytes) {
            this(bytes, 0);
        }

        Reader(byte[] bytes, int pos) {
            this.bytes = bytes;
            this.pos = pos;
        }

        int u1() {
            return bytes[pos++] & 0xFF;
        }

        int u2() {
            int value = u2At(bytes, pos);
            pos += 2;
            return value;
        }

        long u4() {
            long value = ((long) (bytes[pos] & 0xFF) << 24) | ((long) (bytes[pos + 1] & 0xFF) << 16)
                | ((bytes[pos + 2] & 0xFF) << 8) | (bytes[pos + 3] & 0xFF);
            pos += 4;
            return value;
        }

        byte[] bytes(int n) {
            byte[] value = Arrays.copyOfRange(bytes, pos, pos + n);
            pos += n;
            return value;
        }

        void skip(int n) {
            pos += n;
        }
    }

    /**
     * The calls replayed as horizon-world's block_state_data.rs reads them give vanilla's own finished
     * tables: the state of every id (by identity) and the id of every legacy name.
     */
    private static void checkReplay(Class<?> type) throws Exception {
        Dynamic<?>[] map = (Dynamic<?>[]) field(type, "MAP");
        Object2IntMap<String> byName = cast(field(type, "ID_BY_OLD_NAME"));
        var replay = new Dynamic<?>[map.length];
        var defaults = new Dynamic<?>[map.length >> 4];
        var names = new HashMap<String, Integer>();
        for (int i = 0; i < CALLS.size(); i++) {
            int id = IDS.get(i);
            var call = CALLS.get(i);
            replay[id] = call[0];
            if (defaults[id >> 4] == null) defaults[id >> 4] = call[0];
            for (int j = 1; j < call.length; j++) names.putIfAbsent(call[j].get("Name").asString(""), id);
        }
        for (int i = 0; i < replay.length; i++) {
            if (replay[i] == null) replay[i] = defaults[i >> 4];
            if (replay[i] != map[i]) throw new IllegalStateException("BlockStateData replay differs at id " + i);
        }
        if (!names.equals(new HashMap<>(byName))) throw new IllegalStateException("BlockStateData replay differs in ID_BY_OLD_NAME");
    }

    // ------------------------------------------------------------------ fields

    private static Object field(String className, String name) throws Exception {
        return field(Class.forName(className), name);
    }

    private static Object field(Class<?> type, String name) throws Exception {
        Field f = type.getDeclaredField(name);
        f.setAccessible(true);
        return f.get(null);
    }

    @SuppressWarnings("unchecked")
    private static <T> T cast(Object o) {
        return (T) o;
    }

    private static Schema schema(Schema schema, String simpleName) {
        if (!schema.getClass().getSimpleName().equals(simpleName))
            throw new IllegalStateException("expected schema " + simpleName + ", got " + schema.getClass().getName());
        return schema;
    }

    private static Map<String, String> stringMap(String className, String name) throws Exception {
        return new TreeMap<>(VanillaDataFixTablesExporter.<Map<String, String>>nonEmpty(cast(field(className, name)), name));
    }

    private static Set<String> stringSet(String className, String name) throws Exception {
        return new TreeSet<>(VanillaDataFixTablesExporter.<Set<String>>nonEmpty(cast(field(className, name)), name));
    }

    private static Map<Integer, String> intMap(String className, String name) throws Exception {
        Int2ObjectMap<String> map = cast(field(className, name));
        return new TreeMap<>(nonEmpty(map, name));
    }

    // ------------------------------------------------------------------ JSON

    private static JsonArray pair(JsonElement a, JsonElement b) {
        var p = new JsonArray();
        p.add(a);
        p.add(b);
        return p;
    }

    private static void pairs(JsonObject out, String name, Map<String, String> map) {
        var a = new JsonArray();
        map.forEach((k, v) -> a.add(pair(new JsonPrimitive(k), new JsonPrimitive(v))));
        out.add(name, a);
    }

    private static void intPairs(JsonObject out, String name, Map<Integer, String> map) {
        var a = new JsonArray();
        map.forEach((k, v) -> a.add(pair(new JsonPrimitive(k), new JsonPrimitive(v))));
        out.add(name, a);
    }

    private static void strings(JsonObject out, String name, Set<String> set) {
        if (set.isEmpty()) throw new IllegalStateException(name + " is empty");
        var a = new JsonArray();
        set.forEach(a::add);
        out.add(name, a);
    }

    /** The non-null entries of an array indexed by a legacy damage value. */
    private static void indexed(JsonObject out, String name, String[] values) {
        var map = new TreeMap<Integer, String>();
        for (int i = 0; i < values.length; i++) if (values[i] != null) map.put(i, values[i]);
        intPairs(out, name, nonEmpty(map, name));
    }

    private static <T> T nonEmpty(T value, String name) {
        boolean empty = value instanceof Map<?, ?> m ? m.isEmpty() : value instanceof Collection<?> c && c.isEmpty();
        if (value == null || empty) throw new IllegalStateException(name + " is empty");
        return value;
    }
}
