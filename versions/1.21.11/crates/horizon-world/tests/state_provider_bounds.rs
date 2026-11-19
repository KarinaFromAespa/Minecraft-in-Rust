use horizon_world::{
    DATA_VERSION,
    nbt::Tag,
    registry::BlockRegistry,
    worldgen::state_provider::{BlockStateProvider, StateProviderBindings, StateProviderCatalog},
};
use std::collections::{BTreeMap, BTreeSet};
#[test]
fn bound_recursive_providers_and_registry_lists_before_sampling() {
    let registry = BlockRegistry::from_nbt(&Tag::compound([
        ("DataVersion", Tag::Int(DATA_VERSION)),
        (
            "states",
            Tag::List(vec![Tag::compound([(
                "id",
                Tag::String("minecraft:stone".into()),
            )])]),
        ),
        (
            "defaults",
            Tag::compound([("minecraft:stone", Tag::Int(0))]),
        ),
    ]))
    .unwrap();
    let bindings =
        StateProviderBindings::new(&registry, BTreeSet::new(), &BTreeMap::new()).unwrap();
    let mut source = r#"{"Name":"stone"}"#.to_owned();
    for _ in 0..140 {
        source = format!(r#"{{"type":"rotated","state":{source}}}"#);
    }
    assert!(BlockStateProvider::from_json(&source, &bindings).is_err());
    let source = serde_json::json!({"type":"random_block","blocks":vec!["stone";8193]}).to_string();
    assert!(BlockStateProvider::from_json(&source, &bindings).is_err());
    // 1.21.11 has no typed rule-based provider: the rule-based form is a
    // standalone record decoded only by DiskConfiguration. Both typed
    // spellings decode-error; the rule-count and shared work-budget bounds
    // are covered by the record-form unit test in state_provider.rs.
    let rule = serde_json::json!({"if_true":{"type":"true"},"then":{"Name":"stone"}});
    let typed =
        serde_json::json!({"type":"rule_based","fallback":{"Name":"stone"},"rules":vec![rule.clone();1]});
    assert!(BlockStateProvider::from_json(&typed.to_string(), &bindings).is_err());
    let typed = serde_json::json!({"type":"minecraft:rule_based_state_provider","fallback":{"Name":"stone"},"rules":vec![rule;1]});
    assert!(BlockStateProvider::from_json(&typed.to_string(), &bindings).is_err());
    // Content mods' blocks join shared tags: the cap is 2^20 blocks.
    let tags = BTreeMap::from([("horizon:large".into(), vec!["stone".into(); 8193])]);
    assert!(StateProviderBindings::new(&registry, BTreeSet::new(), &tags).is_ok());
    let tags = BTreeMap::from([(
        "horizon:oversized".into(),
        vec!["stone".into(); (1 << 20) + 1],
    )]);
    assert!(StateProviderBindings::new(&registry, BTreeSet::new(), &tags).is_err());
    let tags = BTreeMap::from([("tag".into(), vec![]), ("minecraft:tag".into(), vec![])]);
    assert!(StateProviderBindings::new(&registry, BTreeSet::new(), &tags).is_err());
    let definitions = BTreeMap::from([
        (
            "horizon:cycle".into(),
            r#"{"type":"copy_properties","source":"horizon:cycle"}"#.into(),
        ),
        ("horizon:valid".into(), r#"{"Name":"stone"}"#.into()),
        ("horizon:invalid".into(), r#"{"type":"missing"}"#.into()),
        ("horizon:alias".into(), r#""horizon:valid""#.into()),
    ]);
    let catalog = StateProviderCatalog::from_resources(&definitions).unwrap();
    assert!(catalog.provider(r#""horizon:valid""#, &bindings).is_ok());
    for name in [
        "horizon:cycle",
        "horizon:invalid",
        "horizon:alias",
        "horizon:missing",
    ] {
        assert!(
            catalog
                .provider(&serde_json::to_string(name).unwrap(), &bindings)
                .is_err()
        );
    }
    assert!(
        catalog
            .direct_provider(r#""horizon:valid""#, &bindings)
            .is_err()
    );
    let duplicate = BTreeMap::from([
        ("stone".into(), "{}".into()),
        ("minecraft:stone".into(), "{}".into()),
    ]);
    assert!(StateProviderCatalog::from_resources(&duplicate).is_err());
}
