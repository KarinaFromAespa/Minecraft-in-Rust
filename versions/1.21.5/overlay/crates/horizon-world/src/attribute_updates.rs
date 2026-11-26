//! Consequences of LivingEntity attribute changes, preserving virtual call order.
use crate::living_health::{self, HealthWorld};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdatedAttribute {
    MaxHealth,
    MaxAbsorption,
    Scale,
    Other,
}
pub trait UpdateWorld<H>: HealthWorld {
    fn attribute_kind(&self, attribute: &H) -> UpdatedAttribute;
    fn absorption(&mut self) -> Result<f32, Self::Error>;
    fn refresh_dimensions(&mut self) -> Result<(), Self::Error>;
}
pub fn updated<H, W: UpdateWorld<H>>(world: &mut W, attribute: H) -> Result<(), W::Error> {
    match world.attribute_kind(&attribute) {
        UpdatedAttribute::MaxHealth => {
            let maximum = world.max_health_attribute()? as f32;
            if world.health()? > maximum {
                world.dispatch_set_health(maximum)?;
            }
        }
        UpdatedAttribute::MaxAbsorption => {
            let maximum = world.max_absorption_attribute()? as f32;
            if world.absorption()? > maximum {
                living_health::set_absorption(world, maximum)?;
            }
        }
        UpdatedAttribute::Scale => world.refresh_dimensions()?,
        UpdatedAttribute::Other => {}
    }
    Ok(())
}
