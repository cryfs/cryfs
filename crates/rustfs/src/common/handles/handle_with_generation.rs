use derive_more::Display;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("{handle}.{generation}")]
pub struct HandleWithGeneration<Handle> {
    pub handle: Handle,
    pub generation: u64,
}
