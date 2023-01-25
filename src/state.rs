#[derive(Default, Copy, Clone)]
pub enum RenderMode {
    #[default]
    Standard = 0,
    RealtimeSimulation = 1,
}

impl Into<usize> for RenderMode {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Default, Copy, Clone)]
pub enum Concurrency {
    #[default]
    None = 0,
    ParallelItems = 1,
    ParallelTracks = 2,
    Both = 3,
}

impl Into<usize> for Concurrency {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Default)]
pub struct ForteState {
    // Setup
    pub render_mode: RenderMode,
    pub concurrency: Concurrency,
}
