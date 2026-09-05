use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Default)]
pub struct NotYetExtacted<T: Component> {
    #[reflect(ignore)]
    _dummy: PhantomData<T>,
}
impl<T: Component> Default for NotYetExtacted<T> {
    fn default() -> Self {
        Self { _dummy: default() }
    }
}

