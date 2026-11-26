//! Resident ServerPlayer pending attribute callbacks over its owned live map.
use crate::{
    Result,
    health::Updates,
    motion::PlayerMotion,
};
use horizon_world::{
    attribute_updates::{self, UpdateWorld, UpdatedAttribute},
    attributes::live::{self, Handle, Key, LiveAttributes},
    live_attribute_updates::{self as live_updates, LiveAttributeWorld},
    living_health::{self, HealthWorld},
    object_open_set::CursorError,
};
use std::io;

impl PlayerMotion {
    /// LivingEntity.tick refreshes these callbacks after aiStep and before the
    /// enclosing Player.tick selects its pose. Synchronization stays queued.
    pub fn refresh_attributes(&mut self) -> Result<()> {
        live_updates::refresh(self)
    }
    /// `Player.internalSetAbsorptionAmount`: the amount as given, unclamped
    /// (the saved `AbsorptionAmount`, or what a client's server sent).
    pub fn internal_set_absorption(&mut self, value: f32) -> Result<()> {
        self.store_absorption(value)
    }
}
impl HealthWorld for PlayerMotion {
    type Error = Box<dyn std::error::Error>;
    type Data = ();
    fn health(&mut self) -> Result<f32> {
        Ok(self.health.health())
    }
    fn health_data(&self) {}
    fn max_health_attribute(&mut self) -> Result<f64> {
        self.attribute("minecraft:max_health")
    }
    fn max_absorption_attribute(&mut self) -> Result<f64> {
        self.attribute("minecraft:max_absorption")
    }
    fn store_health(&mut self, _: (), value: f32) -> Result<()> {
        self.health.data.set(
            self.health.ids.health,
            value,
            false,
            &mut Updates(&mut self.events),
        )?;
        Ok(())
    }
    fn store_absorption(&mut self, value: f32) -> Result<()> {
        // Player.internalSetAbsorptionAmount stores the already clamped value
        // through its current synchronized data, without another limit query.
        self.health.data.set(
            self.health.ids.absorption,
            value,
            false,
            &mut Updates(&mut self.events),
        )?;
        Ok(())
    }
    fn dead_flag(&self) -> bool {
        self.health.dead
    }
    fn dispatch_set_health(&mut self, value: f32) -> Result<()> {
        living_health::set_health(self, value)
    }
}
impl UpdateWorld<Key> for PlayerMotion {
    fn attribute_kind(&self, attribute: &Key) -> UpdatedAttribute {
        live_updates::kind(attribute)
    }
    fn absorption(&mut self) -> Result<f32> {
        Ok(self.health.absorption())
    }
    fn refresh_dimensions(&mut self) -> Result<()> {
        PlayerMotion::refresh_dimensions(self)
    }
}
impl LiveAttributeWorld for PlayerMotion {
    type Attributes = ();
    type Identity = Box<dyn horizon_world::attributes::live::Identities>;
    fn get_attributes(&mut self) -> Result<()> {
        Ok(())
    }
    fn live_attributes(&self, _: ()) -> &LiveAttributes<Self::Identity> {
        &self.attributes
    }
    fn live_attributes_mut(&mut self, _: ()) -> &mut LiveAttributes<Self::Identity> {
        &mut self.attributes
    }
    fn instance_attribute(&self, instance: Option<Handle>) -> Result<Key> {
        let instance =
            instance.ok_or_else(|| io::Error::other("null pending attribute instance"))?;
        self.attributes
            .attribute(&instance)
            .cloned()
            .map_err(Self::attribute_error)
    }
    fn attribute_error(error: live::Error) -> Self::Error {
        io::Error::other(format!("resident attribute: {error:?}")).into()
    }
    fn cursor_error(error: CursorError) -> Self::Error {
        io::Error::other(format!("resident attribute cursor: {error:?}")).into()
    }
    fn dispatch_attribute_updated(&mut self, attribute: Key) -> Result<()> {
        // 1.21.5 has no ServerPlayer.onAttributeUpdated waypoint hook (the
        // waypoint attributes arrived in 1.21.6): the inherited
        // LivingEntity handler is the whole dispatch.
        attribute_updates::updated(self, attribute)
    }
}
