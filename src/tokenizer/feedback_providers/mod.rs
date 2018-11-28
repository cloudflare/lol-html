mod ambiguity_guard;
mod tree_builder_simulator;

use self::ambiguity_guard::AmbiguityGuard;
use self::tree_builder_simulator::TreeBuilderSimulator;

pub use self::tree_builder_simulator::TreeBuilderFeedback;

#[derive(Default)]
pub struct FeedbackProviders {
    pub ambiguity_guard: AmbiguityGuard,
    pub tree_builder_simulator: TreeBuilderSimulator,
}
