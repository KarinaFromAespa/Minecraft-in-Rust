//! The two component layouts changed in 1.21.6, from ItemAttributeModifiers.Entry.STREAM_CODEC
//! and Equippable.STREAM_CODEC. Raw values stored in a Stack always use the pinned layout.
use super::*;
use crate::Error;

pub(super) fn read(kind: i32, r: &mut Reader) -> Result<Option<Vec<u8>>> {
    if ids::quirks().components_before_1_21_6 {
        rewrite(kind, r, true)
    } else {
        Ok(None)
    }
}

pub(super) fn write(kind: i32, bytes: &[u8]) -> Option<Vec<u8>> {
    // The target's layout decides: a pre-1.21.6 server, or the pinned version
    // itself when no session is active (this pin predates 1.21.6, so its own
    // wire drops the display and shearing fields too).
    if ids::quirks_written().components_before_1_21_6 {
        rewrite(kind, &mut Reader::new(bytes), false).ok().flatten()
    } else {
        None
    }
}

fn rewrite(kind: i32, r: &mut Reader, up: bool) -> Result<Option<Vec<u8>>> {
    let mut w = Writer::new();
    match kind {
        component::ATTRIBUTE_MODIFIERS => {
            let n = r.count(65536)?;
            w.varint(n as i32);
            for _ in 0..n {
                let attribute = r.varint()?;
                let mapped = match ids::active() {
                    Some((ids, ids::Way::Up)) => ids.attribute.up(attribute),
                    Some((ids, ids::Way::Down)) => ids.attribute.down(attribute),
                    None => Some(attribute),
                }
                .ok_or_else(|| Error::new("attribute absent from target registry"))?;
                w.varint(mapped);
                w.string(&r.string(32767)?);
                w.f64(r.f64()?);
                w.varint(r.varint()?);
                w.varint(r.varint()?);
                if up {
                    w.varint(0); // Display.Default, added in 1.21.6.
                } else {
                    match r.varint()? {
                        0 | 1 => {}
                        2 => {
                            r.nbt()?;
                        }
                        _ => return Err(Error::new("unknown attribute modifier display")),
                    }
                }
            }
        }
        component::EQUIPPABLE => {
            w.varint(r.varint()?);
            sound(r, &mut w)?;
            for _ in 0..2 {
                let present = r.bool()?;
                w.bool(present);
                if present {
                    w.string(&r.string(32767)?);
                }
            }
            let allowed = r.bool()?;
            w.bool(allowed);
            if allowed {
                let n = r.varint()?;
                if n == 0 {
                    w.varint(0);
                    w.string(&r.string(32767)?);
                } else {
                    let count = usize::try_from(n - 1)
                        .map_err(|_| Error::new("negative holder set size"))?;
                    if count > r.remaining() {
                        return Err(Error::new("holder set past the end"));
                    }
                    let entities = (0..count)
                        .map(|_| r.varint().map(ids::entity_type))
                        .collect::<Result<Vec<_>>>()?;
                    let entities: Vec<_> = entities.into_iter().filter(|id| *id >= 0).collect();
                    w.varint(entities.len() as i32 + 1);
                    for entity in entities {
                        w.varint(entity);
                    }
                }
            }
            // The side whose wire is read: the source when reading up, the
            // target when writing down; the pinned version when no session
            // is active (this pin predates 1.21.5, so its wire has no
            // equip-on-interact field either).
            let before_5 = if up {
                ids::quirks().components_before_1_21_5
            } else {
                ids::quirks_written().components_before_1_21_5
            };
            for _ in 0..3 {
                w.bool(r.bool()?);
            }
            if up && before_5 {
                w.bool(false);
            } else {
                let interact = r.bool()?;
                if !before_5 {
                    w.bool(interact);
                }
            }
            if up {
                w.bool(false); // Equippable's default canBeSheared.
                // No item.shears.snip at this pin (a 1.21.5 addition): the
                // sheep-shear sound (1226 pinned) stands in as the shearing
                // sound. (The old value 1256 was stale: 1255 was the
                // skeleton ambient at the 1.21.1 pin, not the shear sound.)
                w.varint(1227);
            } else {
                r.bool()?;
                sound(r, &mut Writer::new())?;
            }
        }
        _ => return Ok(None),
    }
    Ok(Some(w.data))
}

fn sound(r: &mut Reader, w: &mut Writer) -> Result<()> {
    let holder = r.varint()?;
    if holder == 0 {
        w.varint(0);
        w.string(&r.string(32767)?);
        let fixed = r.bool()?;
        w.bool(fixed);
        if fixed {
            w.f32(r.f32()?);
        }
    } else {
        w.varint(ids::sound_event(holder - 1) + 1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::{ServerVersion, ids::Way};

    fn bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn vanilla_1_21_5_component_stacks_match_the_pinned_oracle_and_go_back() {
        // Independent ItemStack.OPTIONAL_STREAM_CODEC output from vanilla 1.21.5 and 26.3.
        // Covers multiple modifiers and a following damage value. The pinned side is the 1.20.6
        // wire form: the item 888 in 1.21.5 is 847 pinned, and the kinds move with the registry
        // (attributes 13 to 12, profile 61 to 46, container 66 to 51). The equippable cases are
        // gone: 1.20.6 predates equippable, so a 1.21.5 server's is dropped on read.
        for (label, old, pinned) in [
            (
                "attributes",
                "01f80601000d02020b6d696e6563726166743a783ff80000000000000001150b6d696e6563726166743a793fd00000000000000200",
                "01cf0601000c02027dd3a41b61e131b09f48d20fb816df220b6d696e6563726166743a783ff8000000000000000111afe7f7d1094f3f22842df4dbe1e6f91c0b6d696e6563726166743a793fd0000000000000020001",
            ),
            (
                "combined",
                "01f80602000d01000b6d696e6563726166743a7a400800000000000000000311",
                "01cf0602000c01003b19bf5a8b6333b9b87df22cc006e3460b6d696e6563726166743a7a40080000000000000000010311",
            ),
            (
                "profile",
                "01f80601003d010b486f72697a6f6e576972650000",
                "01cf0601002e010b486f72697a6f6e576972650000",
            ),
            (
                "container",
                "01f80601004206030100000000000002fa020000",
                "01cf0601003306030100000000000002e4020000",
            )
        ] {
            let old = bytes(old);
            let pinned = bytes(pinned);
            let mut r = Reader::new(&old);
            let stack = {
                let _up = ids::renumbering(ServerVersion::V1_21_5.ids().unwrap(), Way::Up);
                Slot::read(&mut r).unwrap()
            };
            assert_eq!(r.remaining(), 0, "{label}");
            let mut expected = Slot::read(&mut Reader::new(&pinned)).unwrap();
            let mut actual = stack.clone();
            actual
                .as_mut()
                .unwrap()
                .added
                .sort_by_key(|(kind, _)| *kind);
            expected
                .as_mut()
                .unwrap()
                .added
                .sort_by_key(|(kind, _)| *kind);
            // The UUID form cannot carry arbitrary modifier ids: the pinned
            // read of the attribute fixtures yields minecraft:<uuid> where
            // the newer read yields minecraft:x/minecraft:y/minecraft:z. The
            // pinned bytes are still parsed above, and the round-trip below
            // is byte-exact.
            if label != "attributes" && label != "combined" {
                assert_eq!(actual, expected, "{label}");
            }
            let mut w = Writer::new();
            {
                let _down = ids::renumbering(ServerVersion::V1_21_5.ids().unwrap(), Way::Down);
                Slot::write(&stack, &mut w);
            }
            assert_eq!(w.data, old, "{label}");
            // The cached delimited patch must also contain normalized values
            // (except the attribute fixtures: their arbitrary ids are lossy
            // through the pin's UUID-form delimited wire, as above).
            if label != "attributes" && label != "combined" {
                let mut w = Writer::new();
                DelimitedSlot::write(&stack, &mut w);
                let read_back = DelimitedSlot::read(&mut Reader::new(&w.data)).unwrap();
                assert_eq!(read_back, stack, "{label}");
            }
        }
    }
}
