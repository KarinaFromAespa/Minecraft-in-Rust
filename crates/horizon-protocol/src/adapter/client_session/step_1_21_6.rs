//! 1.21.8's packets in 1.21.6's layout for a 1.21.6 client (and 1.21.5): the mirror of `translate`'s `mod v1_21_6`
//! (1.21.6 to 1.21.8). One packet differs, and entity data is laid out alike.
//! - set_equipment: 1.21.8 added the saddle slot (7) to its equipment enum. A 1.21.6 client has no such slot, so
//!   its entries are removed. A packet with nothing else in it is dropped.
//!
//! The test-block packets are gone at the 1.21.4 pin: 1.21.4 has no test_instance_block_action or
//! set_test_block, so the id tables drop them and nothing here needs rewriting.
//! Entity data: no index or type changed between 1.21.6 and 1.21.8. The serializer ids differ only from 1.21.9.
use crate::{
    Result, State,
    buf::{Codec, Reader, Writer},
    ids as packet_ids,
    packets::play,
};

/// What becomes of a payload the 1.21.8 side sent or the client was sent.
#[derive(Debug, PartialEq)]
pub(super) enum Out {
    /// Laid out alike: the payload passes as it is.
    Untouched,
    Rewritten(Vec<u8>),
    /// The 1.21.6 client can't read it: it is not sent (clientbound) or not read (serverbound).
    Dropped,
}

/// EquipmentSlot.SADDLE, new in 1.21.8.
const SADDLE: u8 = 7;

/// The pinned clientbound packet `id` (`payload` in 1.21.8's layout) in 1.21.6's layout.
pub(super) fn clientbound(state: State, id: i32, payload: &[u8]) -> Result<Out> {
    use packet_ids::play::clientbound as cb;
    if (state, id) != (State::Play, cb::SET_EQUIPMENT) {
        return Ok(Out::Untouched);
    }
    let mut e = play::SetEquipment::read(&mut Reader::new(payload))?;
    let before = e.slots.len();
    e.slots.retain(|(slot, _)| *slot != SADDLE);
    if e.slots.len() == before {
        return Ok(Out::Untouched);
    }
    if e.slots.is_empty() {
        return Ok(Out::Dropped);
    }
    let mut w = Writer::new();
    play::SetEquipment::write(&e, &mut w);
    Ok(Out::Rewritten(w.data))
}

/// The pinned serverbound packet `id` as the 1.21.6 client sent it (`payload`) in 26.3's layout, for the server.
/// 1.21.4 has no test-block packets, so nothing here needs rewriting: the id tables drop them.
pub(super) fn serverbound(_state: State, _id: i32, _payload: &[u8]) -> Result<Out> {
    Ok(Out::Untouched)
}

#[cfg(test)]
mod tests {
    //! Each packet written here is read back with `translate`'s 1.21.6 reader (the serverbound one, which
    //! takes the 1.21.8 layout the client sends) or the pinned codec for the clientbound one.
    use super::*;
    use crate::item::Stack;

    #[test]
    fn set_equipment_loses_the_saddle_and_keeps_the_other_slots() {
        let stack = Stack::new(1, 1);
        let p = play::SetEquipment {
            id: 4,
            slots: vec![
                (0, Some(stack.clone())),
                (SADDLE, Some(stack.clone())),
                (5, None),
            ],
        };
        let mut w = Writer::new();
        play::SetEquipment::write(&p, &mut w);
        let Out::Rewritten(body) = clientbound(
            State::Play,
            packet_ids::play::clientbound::SET_EQUIPMENT,
            &w.data,
        )
        .unwrap() else {
            panic!("the saddle should be removed");
        };
        // The top bit of the last entry is clear again: read back, the list is the two others.
        let back = play::SetEquipment::read(&mut Reader::new(&body)).unwrap();
        assert_eq!(
            back,
            play::SetEquipment {
                id: 4,
                slots: vec![(0, Some(stack)), (5, None)],
            }
        );
    }

    #[test]
    fn set_equipment_of_only_a_saddle_is_dropped() {
        let p = play::SetEquipment {
            id: 4,
            slots: vec![(SADDLE, Some(Stack::new(1, 1)))],
        };
        let mut w = Writer::new();
        play::SetEquipment::write(&p, &mut w);
        assert_eq!(
            clientbound(
                State::Play,
                packet_ids::play::clientbound::SET_EQUIPMENT,
                &w.data
            )
            .unwrap(),
            Out::Dropped
        );
    }

    #[test]
    fn set_equipment_without_a_saddle_is_untouched() {
        let p = play::SetEquipment {
            id: 4,
            slots: vec![(6, None)],
        };
        let mut w = Writer::new();
        play::SetEquipment::write(&p, &mut w);
        assert_eq!(
            clientbound(
                State::Play,
                packet_ids::play::clientbound::SET_EQUIPMENT,
                &w.data
            )
            .unwrap(),
            Out::Untouched
        );
    }
}
