//! `TestEnvironmentDefinition`: what a game test batch changes in the world
//! while it runs (game rules, weather, the day time, difficulty, functions,
//! all of them), set up when the batch starts and torn down after it. The
//! definitions are the data pack's `test_environment` entries, or inline.
use super::Game;
use horizon_client::gamerules::{self, Value};
use serde_json::Value as Json;

/// `Weather.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WeatherType {
    Clear,
    Rain,
    Thunder,
}

impl WeatherType {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "clear" => Some(Self::Clear),
            "rain" => Some(Self::Rain),
            "thunder" => Some(Self::Thunder),
            _ => None,
        }
    }

    /// (clearTime, rainTime, raining, thundering).
    fn parameters(self) -> (i32, i32, bool, bool) {
        match self {
            Self::Clear => (100_000, 0, false, false),
            Self::Rain => (0, 100_000, true, false),
            Self::Thunder => (0, 100_000, true, true),
        }
    }
}

/// The default difficulty (`Difficulty.NORMAL`): what a `difficulty`
/// environment resets to on teardown. The type is a Horizon extra — vanilla
/// has no difficulty environment in either version — so it follows the
/// 1.21.10 game-rules pattern of resetting to the default.
const DEFAULT_DIFFICULTY: u8 = 2; // "normal"

/// A `TestEnvironmentDefinition` (1.21.10: no saved data; `setup` and
/// `teardown` take the level only. 26.x's generic `SavedDataType` and
/// `Activation` are gone, with `clock_time` and `timeline_attributes`).
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Environment {
    AllOf(Vec<Environment>),
    Difficulty(u8),
    Functions {
        setup: Option<String>,
        teardown: Option<String>,
    },
    GameRules(Vec<(String, Value)>),
    /// `time_of_day`: `ServerLevel.setDayTime`, with no teardown — the time
    /// stays changed after the batch.
    TimeOfDay {
        time: i64,
    },
    Weather(WeatherType),
}

/// Reads a definition from its JSON (`DIRECT_CODEC`); a string is the id of
/// a registry entry, read by `load`.
pub(super) fn parse(
    json: &Json,
    load: &dyn Fn(&str) -> Option<Json>,
    depth: u32,
) -> Option<Environment> {
    if depth > 16 {
        return None;
    }
    if let Some(id) = json.as_str() {
        return parse(&load(id)?, load, depth + 1);
    }
    let kind = json.get("type")?.as_str()?;
    let kind = kind.strip_prefix("minecraft:").unwrap_or(kind);
    Some(match kind {
        "all_of" => Environment::AllOf(
            json.get("definitions")?
                .as_array()?
                .iter()
                .map(|d| parse(d, load, depth + 1))
                .collect::<Option<_>>()?,
        ),
        "time_of_day" => Environment::TimeOfDay {
            // `ExtraCodecs.NON_NEGATIVE_INT`.
            time: json
                .get("time")?
                .as_i64()
                .filter(|t| (0..=i64::from(i32::MAX)).contains(t))?,
        },
        "difficulty" => Environment::Difficulty(
            ["peaceful", "easy", "normal", "hard"]
                .iter()
                .position(|d| Some(*d) == json.get("difficulty").and_then(Json::as_str))?
                as u8,
        ),
        "function" => Environment::Functions {
            setup: json.get("setup").and_then(Json::as_str).map(str::to_owned),
            teardown: json
                .get("teardown")
                .and_then(Json::as_str)
                .map(str::to_owned),
        },
        "game_rules" => Environment::GameRules(
            json.get("rules")?
                .as_object()?
                .iter()
                .map(|(name, v)| {
                    let rule = gamerules::rule(name)?;
                    let value = match (v.as_bool(), v.as_i64()) {
                        (Some(b), _) => Value::Bool(b),
                        (_, Some(i)) => Value::Int(i32::try_from(i).ok()?),
                        _ => return None,
                    };
                    Some((rule.name.to_owned(), value))
                })
                .collect::<Option<_>>()?,
        ),
        "weather" => Environment::Weather(WeatherType::from_name(json.get("weather")?.as_str()?)?),
        _ => return None,
    })
}

impl Game<'_> {
    /// The data pack entry `data/<ns>/test_environment/<path>.json`.
    fn read_test_environment_json(&self, id: &str) -> Option<Json> {
        let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
        let file = self
            .options
            .resources
            .join("data")
            .join(ns)
            .join("test_environment")
            .join(format!("{path}.json"));
        serde_json::from_slice(&std::fs::read(file).ok()?).ok()
    }

    /// The definition of a batch's environment (`batch()` of its tests).
    pub(super) fn test_environment(&self, id: &str) -> Option<Environment> {
        parse(
            &Json::String(id.to_owned()),
            &|i| self.read_test_environment_json(i),
            0,
        )
    }

    /// `TestEnvironmentDefinition.setup`: the runner keeps the definition
    /// itself (`GameTestRunner.currentEnvironment`) and calls this, then
    /// `tear_down_test_environment`, directly.
    pub(super) fn set_up_test_environment(&mut self, environment: &Environment) {
        self.set_up_environment(environment);
    }

    /// `TestEnvironmentDefinition.teardown`.
    pub(super) fn tear_down_test_environment(&mut self, environment: &Environment) {
        self.tear_down_environment(environment);
    }

    fn set_up_environment(&mut self, environment: &Environment) {
        match environment {
            Environment::AllOf(all) => {
                for e in all {
                    self.set_up_environment(e);
                }
            }
            Environment::TimeOfDay { time } => {
                // `ServerLevel.setDayTime`.
                self.world.day_time = *time;
                self.ui.clock.partial = 0.0;
            }
            Environment::Difficulty(d) => {
                self.set_difficulty_forced(*d);
            }
            Environment::Functions { setup, .. } => {
                if let Some(id) = setup {
                    self.run_environment_function(id);
                }
            }
            Environment::GameRules(rules) => self.set_environment_rules(rules),
            Environment::Weather(kind) => self.set_weather_type(*kind),
        }
    }

    fn tear_down_environment(&mut self, environment: &Environment) {
        match environment {
            // `AllOf.teardown`: in order (1.21.10; 26.x tore down reversed).
            Environment::AllOf(all) => {
                for e in all {
                    self.tear_down_environment(e);
                }
            }
            // `TimeOfDay` has no teardown: the time stays changed.
            Environment::TimeOfDay { .. } => {}
            Environment::Difficulty(_) => self.set_difficulty_forced(DEFAULT_DIFFICULTY),
            Environment::Functions { teardown, .. } => {
                if let Some(id) = teardown {
                    self.run_environment_function(id);
                }
            }
            Environment::GameRules(rules) => {
                // `SetGameRules.teardown`: each touched rule goes back to its
                // default (1.21.10; 26.x restored the pre-test values).
                let defaults: Vec<(String, Value)> = rules
                    .iter()
                    .filter_map(|(name, _)| {
                        let rule = gamerules::rule(name)?;
                        Some((rule.name.to_owned(), rule.default))
                    })
                    .collect();
                self.set_environment_rules(&defaults);
            }
            Environment::Weather(_) => {
                // `Weather.teardown`: `resetWeatherCycle` only (1.21.10; 26.x
                // re-applied the previous weather afterwards).
                let w = &mut self.world.weather;
                w.rain_time = 0;
                w.raining = false;
                w.thunder_time = 0;
                w.thundering = false;
            }
        }
    }

    /// `GameRules.setAll`.
    fn set_environment_rules(&mut self, rules: &[(String, Value)]) {
        for (name, value) in rules {
            if let Some(rule) = gamerules::rule(name) {
                self.ui.rules.set(rule.name, *value);
            }
        }
        self.sync_rules();
    }

    /// `Weather.Type.apply`: `ServerLevel.setWeatherParameters`.
    fn set_weather_type(&mut self, kind: WeatherType) {
        let (clear, rain, raining, thundering) = kind.parameters();
        let w = &mut self.world.weather;
        w.clear_time = clear;
        w.rain_time = rain;
        w.thunder_time = rain;
        w.raining = raining;
        w.thundering = thundering;
    }

    /// `Functions.run`: the function with game master permission and
    /// suppressed output; a missing one is logged.
    fn run_environment_function(&mut self, id: &str) {
        let id = if id.contains(':') {
            id.to_owned()
        } else {
            format!("minecraft:{id}")
        };
        let found = self
            .functions_named(&id)
            .ok()
            .and_then(|f| f.into_iter().next());
        match found {
            Some(function) => {
                let source = crate::game::commands::execute::Source {
                    silent: true,
                    ..self.server_source()
                };
                self.exec.commands = 0;
                let _ = self.run_function(&function, None, source);
            }
            None => log::error!("Test Batch failed for non-existent function {id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn load(id: &str) -> Option<Json> {
        match id {
            "minecraft:night" => Some(json!({"type": "minecraft:time_of_day", "time": 18000})),
            _ => None,
        }
    }

    #[test]
    fn definitions_parse_with_references_and_nesting() {
        let json = json!({
            "type": "minecraft:all_of",
            "definitions": [
                "minecraft:night",
                {"type": "minecraft:weather", "weather": "thunder"},
                {"type": "minecraft:game_rules", "rules": {"minecraft:advance_time": false}},
                {"type": "minecraft:difficulty", "difficulty": "hard"},
                {"type": "minecraft:function", "setup": "test:on"}
            ]
        });
        let Some(Environment::AllOf(all)) = parse(&json, &load, 0) else {
            panic!("not all_of");
        };
        assert_eq!(all[0], Environment::TimeOfDay { time: 18000 });
        assert_eq!(all[1], Environment::Weather(WeatherType::Thunder));
        assert!(matches!(&all[2], Environment::GameRules(r) if r.len() == 1));
        assert_eq!(all[3], Environment::Difficulty(3));
        assert_eq!(
            all[4],
            Environment::Functions {
                setup: Some("test:on".into()),
                teardown: None
            }
        );
    }

    #[test]
    fn unknown_or_invalid_definitions_do_not_parse() {
        assert_eq!(parse(&json!("minecraft:missing"), &load, 0), None);
        assert_eq!(parse(&json!({"type": "minecraft:nothing"}), &load, 0), None);
        assert_eq!(
            parse(
                &json!({"type": "minecraft:time_of_day", "time": -1}),
                &load,
                0
            ),
            None
        );
        assert_eq!(
            parse(
                &json!({"type": "minecraft:time_of_day", "time": 2_147_483_648_i64}),
                &load,
                0
            ),
            None
        );
        // 26.x-only types: no `clock_time`, no `timeline_attributes`.
        assert_eq!(
            parse(
                &json!({"type": "minecraft:clock_time", "clock": "minecraft:overworld", "time": 18000}),
                &load,
                0
            ),
            None
        );
        assert_eq!(
            parse(
                &json!({"type": "minecraft:timeline_attributes", "timelines": []}),
                &load,
                0
            ),
            None
        );
    }

    #[test]
    fn weather_types_set_vanillas_timers() {
        assert_eq!(WeatherType::Clear.parameters(), (100_000, 0, false, false));
        assert_eq!(WeatherType::Thunder.parameters(), (0, 100_000, true, true));
    }

    #[test]
    #[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES and scratch homes/cwd"]
    fn teardown_resets_rules_and_weather_and_leaves_the_time() {
        let mut game = crate::game::native_machines::tests::server_game();
        // GameRules: the pre-test value differs from the default, the
        // environment sets the default; teardown must leave the default
        // (1.21.10 resets to default — 26.x restored the pre-test value).
        let rule = gamerules::rule("minecraft:advance_time").unwrap();
        let Value::Bool(default) = rule.default else {
            panic!("advance_time is a bool rule");
        };
        let before = Value::Bool(!default);
        game.set_environment_rules(&[(rule.name.to_owned(), before)]);
        assert_eq!(game.ui.rules.get(rule.name), Some(before));
        let rules = Environment::GameRules(vec![(rule.name.to_owned(), rule.default)]);
        game.set_up_test_environment(&rules);
        assert_eq!(game.ui.rules.get(rule.name), Some(rule.default));
        game.tear_down_test_environment(&rules);
        assert_eq!(game.ui.rules.get(rule.name), Some(rule.default));
        // Weather: raining before, clear during; teardown only resets the
        // cycle (1.21.10) — it must not bring the rain back.
        game.set_weather_type(WeatherType::Rain);
        assert!(game.world.weather.raining);
        let weather = Environment::Weather(WeatherType::Clear);
        game.set_up_test_environment(&weather);
        assert!(!game.world.weather.raining);
        game.tear_down_test_environment(&weather);
        let w = &game.world.weather;
        assert!(!w.raining && !w.thundering && w.rain_time == 0 && w.thunder_time == 0);
        // TimeOfDay: teardown is a no-op; the time stays changed.
        game.world.day_time = 6000;
        let time = Environment::TimeOfDay { time: 18000 };
        game.set_up_test_environment(&time);
        assert_eq!(game.world.day_time, 18000);
        game.tear_down_test_environment(&time);
        assert_eq!(game.world.day_time, 18000);
        // Difficulty (Horizon extra): teardown resets to normal.
        let difficulty = Environment::Difficulty(3);
        game.set_up_test_environment(&difficulty);
        assert_eq!(game.ui.difficulty, 3);
        game.tear_down_test_environment(&difficulty);
        assert_eq!(game.ui.difficulty, DEFAULT_DIFFICULTY);
    }
}
