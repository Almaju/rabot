//! Layers point one way. If main exposes A and A uses B, B does not use A.
//! <https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies>

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings};

pub struct Cycles;

const HELP: &str = "pick the lower layer and make it stop reaching up: move what both need into a module neither depends on, or have the lower module return its own types and let the upper one wrap them";

impl Check for Cycles {
    fn run(&self, cx: &Context) -> Findings {
        let mut findings = Findings::default();
        for edge in cx.module_graph.cycles_in(&cx.file.path) {
            if let Some(mut diagnostic) =
                cx.diagnostic_at(Rule::ModuleCycle, edge.offset, edge.message.clone())
            {
                diagnostic.help = Some(HELP.to_string());
                findings.diagnostics.push(diagnostic);
            }
        }
        findings
    }
}
