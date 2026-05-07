use stagelx_core::universe::{DmxBuffer, UniverseSet};
use crate::merge::MergeStrategy;

/// Priority-ordered DMX source.
#[derive(Debug)]
pub struct DmxSource {
    pub name: String,
    pub priority: u8,
    pub strategy: MergeStrategy,
    pub universes: UniverseSet,
}

/// Combines multiple DMX sources into a single output universe set.
///
/// Sources are processed in ascending priority order; higher priority wins per strategy.
#[derive(Debug, Default)]
pub struct DmxEngine {
    sources: Vec<DmxSource>,
    output: UniverseSet,
}

impl DmxEngine {
    pub fn add_source(&mut self, source: DmxSource) {
        self.sources.push(source);
        self.sources.sort_by_key(|s| s.priority);
    }

    /// Recompute the output universes from all sources.
    pub fn tick(&mut self) {
        // Collect universe IDs across all sources
        let universe_ids: Vec<u16> = self.sources
            .iter()
            .flat_map(|s| s.universes.universes().map(|(id, _)| id))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for uid in universe_ids {
            let out = self.output.get_or_insert(uid);
            out.clear();
            for source in &self.sources {
                if let Some(buf) = source.universes.get(uid) {
                    match source.strategy {
                        MergeStrategy::Htp => out.merge_htp(buf),
                        MergeStrategy::Ltp => out.merge_ltp(buf),
                    }
                }
            }
        }
    }

    pub fn output(&self) -> &UniverseSet {
        &self.output
    }

    pub fn output_buffer(&self, universe: u16) -> Option<&DmxBuffer> {
        self.output.get(universe)
    }
}
