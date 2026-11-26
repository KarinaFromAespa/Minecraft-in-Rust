package dev.horizonlab.parity;

import java.io.PrintWriter;
import java.nio.file.*;
import java.util.*;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.screens.LevelLoadingScreen;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.sounds.MusicInfo;
import net.minecraft.client.sounds.MusicManager;
import net.minecraft.client.sounds.SoundEngine;
import net.minecraft.client.sounds.SoundManager;
import net.minecraft.core.Holder;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.sounds.Music;
import net.minecraft.sounds.SoundEvent;
import net.minecraft.sounds.SoundSource;
import net.minecraft.util.RandomSource;

/**
 * Drives the real MusicManager over the real SoundEngine (VanillaSoundRig).
 * The situation, Minecraft.getSituationalMusic, is the script's input; the
 * loading screen is a LevelLoadingScreen in the Gui. Each client tick is
 * musicManager.tick() then soundManager.tick(pause), as Minecraft.tick
 * runs them. Prints the manager's state and the music channel every step.
 */
public final class VanillaMusicOracle {
    public static final class Mc extends Minecraft {
        Music music;
        float volume = 1.0F;

        Mc() {
            super(null);
        }

        @Override
        public MusicInfo getSituationalMusic() {
            return new MusicInfo(music, volume);
        }
    }

    static Music music(String spec) {
        if (spec.equals("-")) return null;
        String[] p = spec.split(",");
        return new Music(Holder.direct(SoundEvent.createVariableRangeEvent(ResourceLocation.parse(p[0]))), Integer.parseInt(p[1]),
            Integer.parseInt(p[2]), p[3].equals("1"));
    }

    static float f(String hex) {
        return Float.intBitsToFloat(Integer.parseUnsignedInt(hex, 16));
    }

    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        var rig = new VanillaSoundRig(Path.of(args[0]));
        Mc mc = VanillaSoundRig.make(Mc.class);
        Gui gui = VanillaSoundRig.make(Gui.class);
        VanillaSoundRig.set(mc, Minecraft.class, "gui", gui);
        VanillaSoundRig.set(mc, Minecraft.class, "soundManager", rig.manager);
        var loading = VanillaSoundRig.make(LevelLoadingScreen.class);
        MusicManager manager = null;
        var gains = VanillaSoundRig.field(SoundEngine.class, "gainBySource");
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[2])))) {
            for (String line : Files.readAllLines(Path.of(args[1]))) {
                out.println(line);
                String[] a = line.split(" ");
                switch (a[0]) {
                    case "start" -> {
                        // A fresh manager: MusicManager(minecraft) with a seeded random.
                        manager = VanillaSoundRig.make(MusicManager.class);
                        VanillaSoundRig.set(manager, MusicManager.class, "minecraft", mc);
                        VanillaSoundRig.set(manager, MusicManager.class, "random", RandomSource.create(Long.parseLong(a[1])));
                        VanillaSoundRig.set(manager, MusicManager.class, "currentGain", 1.0F);
                        VanillaSoundRig.set(manager, MusicManager.class, "nextSongDelay", 100);
                        rig.manager.stop();
                    }
                    case "situation" -> {
                        mc.music = music(a[1]);
                        mc.volume = f(a[2]);
                    }
                    case "tick" -> {
                        VanillaSoundRig.set(gui, Gui.class, "screen", a[1].equals("1") ? loading : null);
                        manager.tick();
                        rig.manager.tick(a[2].equals("1"));
                    }
                    case "finish" -> {
                        var current = (SoundInstance) VanillaSoundRig.get(manager, MusicManager.class, "currentMusic");
                        if (current != null) rig.finish(current);
                    }
                    case "stop" -> manager.stopPlaying();
                    case "stopmusic" -> manager.stopPlaying(music(a[1]));
                    case "volume" -> {
                        rig.volume(SoundSource.values()[Integer.parseInt(a[1])], f(a[2]));
                        rig.manager.refreshCategoryVolume(SoundSource.values()[Integer.parseInt(a[1])]);
                    }
                    case "stopall" -> rig.manager.stop();
                    default -> throw new IllegalArgumentException(line);
                }
                var current = (SoundInstance) VanillaSoundRig.get(manager, MusicManager.class, "currentMusic");
                out.printf("  current=%s delay=%d gain=%08x musicGain=%08x%n", current == null ? "-" : current.getIdentifier(),
                    (int) VanillaSoundRig.get(manager, MusicManager.class, "nextSongDelay"),
                    Float.floatToRawIntBits((float) VanillaSoundRig.get(manager, MusicManager.class, "currentGain")),
                    Float.floatToRawIntBits(((it.unimi.dsi.fastutil.objects.Object2FloatMap<SoundSource>) gains.get(rig.engine)).getFloat(SoundSource.MUSIC)));
                if (current != null) out.println(rig.describe("music", current));
                rig.flush(out);
            }
        }
        System.exit(0);
    }
}
