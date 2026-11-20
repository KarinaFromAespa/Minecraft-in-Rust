//! A recipe with several displays still has no ghost (PlaceRecipe is
//! unchanged); an ordinary recipe's ghost is still sent. (The book-add
//! half of this file is gone with the pin's recipe-book sync: the hub
//! no longer sends the combined `recipe` packet yet.)
use super::*;
use horizon_client::{
    chunk::Chunk,
    lan::{Joined, NetConfig, PlayerSender, guest::Guest, hub::HubConfig},
    motion::PlayerMotion,
};
use horizon_protocol::packets::configuration::ClientInformation;
use std::sync::{Arc, atomic::AtomicI32, mpsc};

const GUEST: &str = "RecipeGuest";

/// A host on a stone floor with one connected, placed guest (conn 2).
fn game() -> (Game<'static>, mpsc::Receiver<horizon_client::lan::Out>) {
    let root = std::path::PathBuf::from(
        std::env::var_os("HORIZON_TEST_RESOURCES")
            .expect("set HORIZON_TEST_RESOURCES to prepared resources"),
    );
    let settings = Settings::default();
    let loaded = load_resources(
        root.clone(),
        None,
        &root,
        &settings,
        std::time::Instant::now(),
        false,
        true,
        &Progress::new(),
        &mut |_, _| unreachable!(),
    )
    .unwrap();
    let mut options = Options::parse(std::iter::empty()).unwrap().unwrap();
    options.resources = root.clone();
    options.execution_role = execution_role::ExecutionRole::IntegratedServer;
    let res: &'static _ = Box::leak(Box::new(loaded.resources));
    let mut game = Game::new(
        res,
        loaded.pack,
        loaded.gui.unwrap(),
        options,
        settings,
        Audio::silent(&root),
        None,
    )
    .unwrap();
    game.build_registries();
    game.set_allow_commands(true);
    for x in 0..2 {
        game.world
            .chunks
            .insert((x, 0), Arc::new(Chunk::empty(x, 0)));
    }
    assert!(game.run_command("fill 0 99 0 31 99 15 stone"));
    game.player = Some(PlayerMotion::new(game.motion, Vec3::new(28.5, 100.0, 10.5)).unwrap());
    let net = NetConfig {
        announce: false,
        ..NetConfig::lan(0, "recipe-variants-test".into(), 2)
    };
    game.lan_host(
        super::lan::LanSettings {
            port: 0,
            guest_commands: true,
            force_game_mode: false,
        },
        net,
        HubConfig {
            default_game_mode: 1,
            guest_commands: true,
            force_game_mode: false,
            hardcore: false,
            view_distance: 2,
            simulation_distance: 2,
            dimension: Dim::Overworld.id().into(),
            allow_flight: false,
            seed: 0,
            max_players: 2,
            integrated: false,
            levels: Default::default(),
        },
    )
    .unwrap();
    let (tx, rx) = mpsc::channel();
    let mut guest = Guest::new(
        Joined {
            conn: 2,
            uuid: 2,
            name: GUEST.into(),
            properties: Vec::new(),
            info: ClientInformation::new(2),
            brand: "test".into(),
            sender: PlayerSender::new(tx, Arc::default()),
            latency: Arc::new(AtomicI32::new(0)),
            address: "127.0.0.1:25565".parse().unwrap(),
            expected: false,
            configured: Default::default(),
        },
        2,
        [12.5, 100.0, 10.5],
        1,
    );
    guest.placed = true;
    guest.loaded = true;
    // Its view, as the client sent it: straight down.
    guest.pitch = 90.0;
    game.lan.as_mut().unwrap().hub.guests.push(guest);
    (game, rx)
}

// Added after the baseline (the authorized adapter): a recipe with several
// displays still has no ghost (PlaceRecipe is unchanged), and an ordinary
// recipe's ghost is still sent.
fn ghosts(rx: &mpsc::Receiver<horizon_client::lan::Out>) -> usize {
    use horizon_protocol::packets::play::Clientbound as C;
    let mut n = 0;
    for out in rx.try_iter() {
        let bodies = match out {
            horizon_client::lan::Out::Packet(body) => vec![body],
            horizon_client::lan::Out::Packets(bodies) => bodies,
            horizon_client::lan::Out::Disconnect(_) => panic!("unexpected disconnect"),
        };
        for body in bodies {
            let mut reader = horizon_protocol::buf::Reader::new(&body);
            let id = reader.varint().unwrap();
            if let Ok(C::PlaceGhostRecipe(_)) = C::decode(id, reader.rest()) {
                n += 1;
            }
        }
    }
    n
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn control_a_multi_display_recipe_still_has_no_ghost() {
    let (mut game, rx) = game();
    let _ = ghosts(&rx);
    let hub = &mut game.lan.as_mut().unwrap().hub;
    hub.send_ghost_recipe(2, 0, "minecraft:map_cloning");
    assert_eq!(ghosts(&rx), 0, "no ghost for eight displays");
    let hub = &mut game.lan.as_mut().unwrap().hub;
    hub.send_ghost_recipe(2, 0, "minecraft:crafting_table");
    assert_eq!(ghosts(&rx), 1, "the ordinary recipe's ghost");
}
