mod render;
pub use render::*;

mod synth;
pub use synth::*;

mod about;
pub use about::*;

#[derive(Default, Copy, Clone, PartialEq)]
pub enum ForteTab {
    Render,
    #[default]
    Synth,
    About,
}
