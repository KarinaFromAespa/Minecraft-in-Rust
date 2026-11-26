use super::*;
use serde_json::json;

#[test]
fn identifier_adds_default_namespace() {
    assert_eq!(identifier("stone").unwrap(), "minecraft:stone");
    assert_eq!(identifier("a:b").unwrap(), "a:b");
}

#[test]
fn positive_modulo_wraps_negatives() {
    assert!((positive_modulo(-0.25, 1.0) - 0.75).abs() < 1e-6);
    assert!((positive_modulo(1.5, 1.0) - 0.5).abs() < 1e-6);
}

#[test]
fn display_context_names_round_trip() {
    for name in ["gui", "ground", "fixed", "head"] {
        assert_eq!(DisplayContext::by_name(name).unwrap().name(), name);
    }
}

#[test]
fn a_mods_tint_source_is_untinted_and_an_unknown_vanilla_one_fails() {
    let tint = |json: &str| TintSource::decode(&serde_json::from_str(json).unwrap());
    assert_eq!(
        tint(r#"{"type":"ae2:storage_cell_state"}"#).unwrap(),
        TintSource::Constant(-1)
    );
    assert!(tint(r#"{"type":"minecraft:nonsense"}"#).is_err());
}

#[test]
fn custom_condition_is_evaluated_per_stack_at_branch_selection() {
    struct Env;
    impl ItemEnv for Env {
        fn item_model_property(&self, property: &str, stack: &StackView) -> bool {
            assert_eq!(property, "example:charged");
            stack.count > 1 && stack.patch.get("example:charged") == Some(&Some(Value::Bool(true)))
        }
    }
    let definition = ClientItem::from_json(
        r#"{"model":{
        "type":"minecraft:condition", "property":"example:charged",
        "on_true":{"type":"minecraft:model","model":"example:item/charged"},
        "on_false":{"type":"minecraft:model","model":"example:item/idle"}
    }}"#,
    )
    .unwrap();
    let mut models = ItemModels::default();
    models.definitions.insert("example:tool".into(), definition);
    models.prototypes.insert(
        "example:tool".into(),
        BTreeMap::from([(
            "minecraft:item_model".into(),
            Value::String("example:tool".into()),
        )]),
    );
    let context = Context {
        display: DisplayContext::Gui,
        owner: None,
        level: None,
        seed: 0,
    };
    let mut stack = StackView::new("example:tool", 1);
    let chosen = |stack: &StackView, env: &dyn ItemEnv| {
        let resolved = models.resolve(stack, &context, env);
        assert_eq!(resolved.layers.len(), 1);
        let LayerContent::Model { model } = &resolved.layers[0].content else {
            panic!("expected model leaf")
        };
        model.clone()
    };
    assert_eq!(chosen(&stack, &NoEnv), "example:item/idle");
    assert_eq!(chosen(&stack, &Env), "example:item/idle");
    stack
        .patch
        .insert("example:charged".into(), Some(Value::Bool(true)));
    assert_eq!(chosen(&stack, &Env), "example:item/idle");
    stack.count = 2;
    assert_eq!(chosen(&stack, &Env), "example:item/charged");
    assert_eq!(chosen(&stack, &NoEnv), "example:item/idle");
    stack.patch.insert("example:charged".into(), None);
    assert_eq!(chosen(&stack, &Env), "example:item/idle");
}

#[test]
fn custom_condition_validates_its_name_and_both_branches() {
    let node = |property: &str, on_true: Value| {
        ItemModelNode::decode(&serde_json::json!({
            "type": "minecraft:condition", "property": property,
            "on_true": on_true, "on_false": {"type": "minecraft:empty"}
        }))
    };
    let empty = serde_json::json!({"type": "minecraft:empty"});
    for name in [
        "minecraft:unknown",
        "unknown",
        "example:Bad",
        "bad name:charged",
    ] {
        assert!(node(name, empty.clone()).is_err(), "{name}");
    }
    assert!(node("example:charged", Value::Null).is_err());
    assert!(matches!(
        node("minecraft:damaged", empty.clone()).unwrap(),
        ItemModelNode::Condition {
            property: ConditionProperty::Damaged,
            ..
        }
    ));
    assert!(matches!(node("example:charged", empty).unwrap(),
        ItemModelNode::Condition { property: ConditionProperty::Custom(name), .. } if name == "example:charged"));
}

#[test]
fn data_component_types_match_the_1_21_11_registry() {
    // `BuiltInRegistries.DATA_COMPONENT_TYPE` in 1.21.11 registers 104
    // types; the item model harness compares this list, in order, against
    // the oracle's sorted registry dump.
    assert_eq!(DATA_COMPONENT_TYPES.len(), 104);
    assert!(DATA_COMPONENT_TYPES.is_sorted());
    for present in ["minecraft:map_color", "minecraft:swing_animation"] {
        assert!(
            DATA_COMPONENT_TYPES.contains(&present),
            "{present} is a 1.21.11 type"
        );
    }
    for absent in [
        "minecraft:attack_animation",
        "minecraft:compostable",
        "minecraft:sign_text_front",
        "minecraft:sulfur_cube_content",
        "minecraft:waxed",
    ] {
        assert!(
            !DATA_COMPONENT_TYPES.contains(&absent),
            "{absent} is not a 1.21.11 type"
        );
    }
}

#[test]
fn a_patched_stack_reads_its_patch_over_its_items_defaults() {
    let mut models = ItemModels::default();
    models.prototypes.insert(
        "minecraft:diamond_sword".into(),
        Components::from_iter([
            (
                "minecraft:item_model".into(),
                json!("minecraft:diamond_sword"),
            ),
            ("minecraft:max_damage".into(), json!(1561)),
            ("minecraft:rarity".into(), json!("common")),
        ]),
    );
    let mut stack = StackView::new("minecraft:diamond_sword", 1);
    stack.patch.insert(
        "minecraft:enchantments".into(),
        Some(json!({"minecraft:sharpness": 5})),
    );
    stack
        .patch
        .insert("minecraft:rarity".into(), Some(json!("rare")));
    stack.patch.insert("minecraft:max_damage".into(), None);
    let view = models.components_view(&stack);
    assert_eq!(view.get("minecraft:rarity"), Some(&json!("rare")));
    assert_eq!(view.get("minecraft:max_damage"), None, "a removed default");
    assert_eq!(
        view.get("minecraft:item_model"),
        Some(&json!("minecraft:diamond_sword"))
    );
    assert!(view.contains_key("minecraft:enchantments"));
    assert!(!view.contains_key("minecraft:custom_name"));
    // Every key reads as the merged map has it.
    let merged = models.components(&stack);
    for key in [
        "minecraft:rarity",
        "minecraft:max_damage",
        "minecraft:item_model",
        "minecraft:enchantments",
        "minecraft:custom_name",
    ] {
        assert_eq!(view.get(key), merged.get(key), "{key}");
    }
    assert_eq!(view.to_map(), *merged);
}

#[test]
fn a_26x_transformation_field_parses_ignored_on_every_unbaked_record() {
    // 1.21.11 has no per-model `transformation` (26.x added it to every
    // Unbaked record and a `Transformation.compose` at bake): DFU ignores
    // the unknown field, so a pack carrying it still parses — and resolves
    // exactly as the same model without it.
    let leaf = json!({"type": "minecraft:model", "model": "example:item/leaf"});
    let transform = json!({"translation": [0, 1, 0]});
    let node = |v: Value| ItemModelNode::decode(&v).unwrap();
    assert!(matches!(
        node(json!({"type": "minecraft:composite", "transformation": transform,
            "models": [leaf.clone()]})),
        ItemModelNode::Composite { models } if models.len() == 1
    ));
    assert!(matches!(
        node(json!({"type": "minecraft:composite", "models": []})),
        ItemModelNode::Composite { models } if models.is_empty()
    ));
    assert!(matches!(
        node(
            json!({"type": "minecraft:condition", "property": "minecraft:damaged",
            "transformation": transform,
            "on_true": leaf.clone(), "on_false": {"type": "minecraft:empty"}})
        ),
        ItemModelNode::Condition { .. }
    ));
    assert!(matches!(
        node(json!({"type": "minecraft:select", "property": "minecraft:main_hand",
            "transformation": transform,
            "cases": [{"when": "right", "model": leaf.clone()}]})),
        ItemModelNode::Select { cases, .. } if cases.len() == 1
    ));
    assert!(matches!(
        node(
            json!({"type": "minecraft:special", "base": "example:item/base",
            "transformation": transform,
            "model": {"type": "minecraft:conduit"}})
        ),
        ItemModelNode::Special { .. }
    ));
    // The resolved layers are identical with and without the field.
    let resolve = |model: &str| {
        let mut models = ItemModels::default();
        models.definitions.insert(
            "example:tool".into(),
            ClientItem::from_json(&format!(r#"{{"model":{model}}}"#)).unwrap(),
        );
        models.prototypes.insert(
            "example:tool".into(),
            BTreeMap::from([(
                "minecraft:item_model".into(),
                Value::String("example:tool".into()),
            )]),
        );
        let context = Context {
            display: DisplayContext::Gui,
            owner: None,
            level: None,
            seed: 0,
        };
        models.resolve(&StackView::new("example:tool", 1), &context, &NoEnv)
    };
    let plain = r#"{"type":"minecraft:composite","models":[
        {"type":"minecraft:model","model":"example:item/leaf"}]}"#;
    let moved = r#"{"type":"minecraft:composite",
        "transformation":{"translation":[0,1,0]},"models":[
        {"type":"minecraft:model","model":"example:item/leaf"}]}"#;
    assert_eq!(resolve(plain), resolve(moved));
    assert_eq!(resolve(plain).layers.len(), 1);
    assert_eq!(
        resolve(r#"{"type":"minecraft:composite","models":[]}"#)
            .layers
            .len(),
        0,
        "an empty composite wraps, it does not substitute the missing model"
    );
}
