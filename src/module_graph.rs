//! Which modules of a crate depend on which, and the cycles among them.
//!
//! A crate's modules are found the way rustc finds them: from the crate root
//! (`src/lib.rs`, `src/main.rs`, `src/bin/*.rs`), following `mod x;` to
//! `x.rs` or `x/mod.rs`. Files outside the checked set are read from disk so
//! a cycle is seen even when only one side of it changed.
//!
//! A dependency is a path that reaches into another module of the same
//! crate: `use crate::a::B`, `super::B`, `self::a::B`, or `a::B` where `a`
//! is a child module. Re-exports (`pub use`, globs included) are followed to
//! where the item is defined, and are not dependencies themselves unless the
//! module's own code uses the name: exposing a module is not depending on it.
//! Test code is left out.
//!
//! Dependencies are compared between siblings. Two modules are siblings under
//! their nearest common ancestor, and that ancestor's own items (its
//! structs, functions, aliases) are a sibling of its children: a crate root
//! that uses `worker`, while `worker` uses a type alias defined in the crate
//! root, is a cycle.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::rule::Rule;
use crate::source_file::SourceFile;

/// Nesting of re-exports followed before giving up.
const MAX_DEPTH: usize = 16;

/// The cycles found across the crates of a run, keyed by the file holding
/// the dependency that closes each one.
#[derive(Clone, Debug, Default)]
pub struct ModuleGraph {
    cycles: BTreeMap<PathBuf, Vec<CycleEdge>>,
}

/// One dependency that is part of a cycle, where it is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleEdge {
    pub message: String,
    /// Byte offset of the path in its file.
    pub offset: usize,
}

/// A module, by its index among the modules of a run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ModuleId(usize);

/// Where a path lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Resolution {
    /// A module of this crate: the one that defines the item, or the module
    /// itself when the path names a module.
    Module(ModuleId),
    /// Another crate, a prelude item, or a local binding.
    Outside,
}

/// A node of the sibling graph under one module: that module's own items,
/// or one of its children (with everything below it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    Child(ModuleId),
    Own,
}

#[derive(Clone, Debug)]
struct Import {
    /// Carries a documented exception: resolves names, is no dependency.
    allowed: bool,
    /// The name the import binds, `None` for a glob or `as _`.
    binding: Option<String>,
    /// Whether the import is visible outside its module: a re-export.
    exported: bool,
    offset: usize,
    segments: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct Module {
    children: BTreeMap<String, ModuleId>,
    defined: BTreeSet<String>,
    file: PathBuf,
    /// First segments of every path the module's own code mentions.
    idents: BTreeSet<String>,
    imports: Vec<Import>,
    parent: Option<ModuleId>,
    path: Vec<String>,
    references: Vec<Reference>,
    /// The crate root this module belongs to.
    root: ModuleId,
}

/// A path in code (outside `use`) that may reach another module.
#[derive(Clone, Debug)]
struct Reference {
    offset: usize,
    segments: Vec<String>,
}

/// A dependency between two sibling nodes, where it is first written.
#[derive(Clone, Debug)]
struct Site {
    file: PathBuf,
    offset: usize,
    text: String,
}

/// The dependencies among the siblings under one module.
#[derive(Default)]
struct Siblings {
    edges: BTreeMap<(Node, Node), Site>,
}

/// The modules of every crate touched by a run.
#[derive(Default)]
struct Crates {
    modules: Vec<Module>,
    owned: BTreeSet<PathBuf>,
}

impl std::ops::Index<ModuleId> for Vec<Module> {
    type Output = Module;

    fn index(&self, id: ModuleId) -> &Module {
        &self[id.0]
    }
}

impl std::ops::IndexMut<ModuleId> for Vec<Module> {
    fn index_mut(&mut self, id: ModuleId) -> &mut Module {
        &mut self[id.0]
    }
}

impl ModuleGraph {
    /// Every module cycle in the crates of `files`.
    pub fn build(files: &[SourceFile]) -> Self {
        let checked: BTreeMap<&Path, &SourceFile> =
            files.iter().map(|file| (file.path.as_path(), file)).collect();
        let mut crates = Crates::default();
        // Files parsed from disk because they are not in the checked set.
        let mut extra = BTreeMap::new();
        let mut packages_done = BTreeSet::new();
        for file in files {
            if let Some(package) = package_of(&file.path)
                && packages_done.insert(package.clone())
            {
                for root in crate_roots(&package) {
                    crates.load(&root, &checked, &mut extra);
                }
            }
        }
        for file in files {
            if !crates.owned.contains(&file.path) {
                crates.load(&file.path, &checked, &mut extra);
            }
        }
        let mut members: BTreeMap<ModuleId, Vec<ModuleId>> = BTreeMap::new();
        for (id, module) in crates.modules.iter().enumerate() {
            members.entry(module.root).or_default().push(ModuleId(id));
        }
        let mut graph = Self::default();
        for modules in members.values() {
            crates.find_cycles(modules, &mut graph);
        }
        for edges in graph.cycles.values_mut() {
            edges.sort_by_key(|edge| edge.offset);
        }
        graph
    }

    /// The cycle dependencies written in `path`.
    pub fn cycles_in(&self, path: &Path) -> &[CycleEdge] {
        self.cycles.get(path).map(Vec::as_slice).unwrap_or_default()
    }
}

impl Crates {
    fn ancestor_below(&self, module: ModuleId, ancestor: ModuleId) -> ModuleId {
        let mut current = module;
        while let Some(parent) = self.modules[current].parent {
            if parent == ancestor {
                return current;
            }
            current = parent;
        }
        current
    }

    /// `module` and its ancestors, nearest first.
    fn chain(&self, module: ModuleId) -> Vec<ModuleId> {
        let mut chain = vec![module];
        let mut current = module;
        while let Some(parent) = self.modules[current].parent {
            chain.push(parent);
            current = parent;
        }
        chain
    }

    fn common_ancestor(&self, a: ModuleId, b: ModuleId) -> ModuleId {
        let chain: BTreeSet<ModuleId> = self.chain(a).into_iter().collect();
        self.chain(b)
            .into_iter()
            .find(|module| chain.contains(module))
            .unwrap_or(a)
    }

    /// Every path of `module` that counts as a dependency: its private
    /// imports, the re-exports its own code uses, and qualified paths.
    fn dependencies(&self, module: ModuleId) -> Vec<(Vec<String>, usize)> {
        let module = &self.modules[module];
        let imports = module
            .imports
            .iter()
            .filter(|import| !import.allowed)
            .filter(|import| {
                !import.exported
                    || import
                        .binding
                        .as_ref()
                        .is_some_and(|binding| module.idents.contains(binding))
            });
        imports
            .map(|import| (import.segments.clone(), import.offset))
            .chain(
                module
                    .references
                    .iter()
                    .map(|reference| (reference.segments.clone(), reference.offset)),
            )
            .collect()
    }

    fn describe(&self, node: Node, level: ModuleId) -> String {
        let module = match node {
            Node::Child(module) => module,
            Node::Own => level,
        };
        let path = &self.modules[module].path;
        match (node, path.is_empty()) {
            (Node::Own, true) => "the crate root".to_string(),
            (Node::Own, false) => format!("`crate::{}` itself", path.join("::")),
            (Node::Child(_), _) => format!("`crate::{}`", path.join("::")),
        }
    }

    /// Group every dependency among the modules of one crate by the level
    /// at which it connects two siblings, then report the ones that close a
    /// cycle.
    fn find_cycles(&self, members: &[ModuleId], graph: &mut ModuleGraph) {
        let mut levels: BTreeMap<ModuleId, Siblings> = BTreeMap::new();
        for &module in members {
            for (segments, offset) in self.dependencies(module) {
                let Resolution::Module(target) = self.resolve(module, &segments, 0) else {
                    continue;
                };
                if target == module {
                    continue;
                }
                let level = self.common_ancestor(module, target);
                let node = |end: ModuleId| {
                    if end == level {
                        Node::Own
                    } else {
                        Node::Child(self.ancestor_below(end, level))
                    }
                };
                let edge = (node(module), node(target));
                if edge.0 == edge.1 {
                    continue;
                }
                levels
                    .entry(level)
                    .or_default()
                    .edges
                    .entry(edge)
                    .or_insert_with(|| Site {
                        file: self.modules[module].file.clone(),
                        offset,
                        text: segments.join("::"),
                    });
            }
        }
        for (level, siblings) in &levels {
            for ((from, to), site) in &siblings.edges {
                let Some(back) = siblings.shortest_path(*to, *from) else {
                    continue;
                };
                let (start, next) = (self.describe(*from, *level), self.describe(*to, *level));
                let message = if back.len() == 2 {
                    format!(
                        "{start} depends on {next} (`{}`), and {next} depends on {start}: neither is the lower layer",
                        site.text,
                    )
                } else {
                    let mut cycle = vec![start.clone()];
                    cycle.extend(back.iter().map(|node| self.describe(*node, *level)));
                    format!(
                        "{start} depends on {next} (`{}`), which leads back to it: {}",
                        site.text,
                        cycle.join(" -> "),
                    )
                };
                graph
                    .cycles
                    .entry(site.file.clone())
                    .or_default()
                    .push(CycleEdge {
                        message,
                        offset: site.offset,
                    });
            }
        }
    }

    /// Parse `path` (or take it from the checked set) and every module file
    /// it declares, as the root of a new crate.
    fn load(
        &mut self,
        path: &Path,
        checked: &BTreeMap<&Path, &SourceFile>,
        extra: &mut BTreeMap<PathBuf, SourceFile>,
    ) {
        if self.owned.contains(path) {
            return;
        }
        let id = ModuleId(self.modules.len());
        self.modules.push(Module {
            file: path.to_path_buf(),
            root: id,
            ..Module::default()
        });
        self.load_file(id, path, true, checked, extra);
    }

    /// Fill `module` from the file at `path`, then load the files of the
    /// modules it declares. `mod_rs` is true for files whose child modules
    /// sit next to them (crate roots and `mod.rs`).
    fn load_file(
        &mut self,
        module: ModuleId,
        path: &Path,
        mod_rs: bool,
        checked: &BTreeMap<&Path, &SourceFile>,
        extra: &mut BTreeMap<PathBuf, SourceFile>,
    ) {
        if !self.owned.insert(path.to_path_buf()) {
            return;
        }
        self.modules[module].file = path.to_path_buf();
        if !checked.contains_key(path) && !extra.contains_key(path) {
            let Ok(text) = std::fs::read_to_string(path) else {
                return;
            };
            let Ok(file) = SourceFile::parse(path, text) else {
                return;
            };
            extra.insert(path.to_path_buf(), file);
        }
        let Some(file) = checked.get(path).copied().or_else(|| extra.get(path)) else {
            return;
        };
        let dir = match (mod_rs, path.parent(), path.file_stem()) {
            (true, Some(parent), _) => parent.to_path_buf(),
            (false, Some(parent), Some(stem)) => parent.join(stem),
            _ => return,
        };
        let mut collector = Collector {
            crates: self,
            declared: Vec::new(),
            dirs: vec![dir],
            file,
            stack: vec![module],
        };
        collector.visit_file(&file.ast);
        let declared = collector.declared;
        for (child, candidates) in declared {
            if let Some(found) = candidates.iter().find(|candidate| candidate.0.is_file()) {
                self.load_file(child, &found.0, found.1, checked, extra);
            }
        }
    }

    fn resolve(&self, from: ModuleId, segments: &[String], depth: usize) -> Resolution {
        if depth > MAX_DEPTH {
            return Resolution::Outside;
        }
        let Some(first) = segments.first() else {
            return Resolution::Outside;
        };
        let (mut current, mut rest) = match first.as_str() {
            "crate" => (self.modules[from].root, &segments[1..]),
            "self" => (from, &segments[1..]),
            "super" => {
                let mut current = from;
                let mut rest = segments;
                while rest.first().is_some_and(|segment| segment == "super") {
                    let Some(parent) = self.modules[current].parent else {
                        return Resolution::Outside;
                    };
                    current = parent;
                    rest = &rest[1..];
                }
                (current, rest)
            }
            name if self.modules[from].children.contains_key(name) => (from, segments),
            _ => return Resolution::Outside,
        };
        while let Some(child) = rest
            .first()
            .and_then(|name| self.modules[current].children.get(name))
        {
            current = *child;
            rest = &rest[1..];
        }
        match rest.first() {
            None => Resolution::Module(current),
            Some(name) => self
                .resolve_name(current, name, depth + 1)
                .unwrap_or(Resolution::Module(current)),
        }
    }

    /// Where the item `name`, as seen from inside `module`, is defined:
    /// here, behind a named import, or behind a glob. `None` when the
    /// module does not know the name.
    fn resolve_name(&self, module: ModuleId, name: &str, depth: usize) -> Option<Resolution> {
        if depth > MAX_DEPTH {
            return None;
        }
        let this = &self.modules[module];
        if this.defined.contains(name) {
            return Some(Resolution::Module(module));
        }
        if let Some(import) = this
            .imports
            .iter()
            .find(|import| import.binding.as_deref() == Some(name))
        {
            return Some(self.resolve(module, &import.segments, depth + 1));
        }
        this.imports
            .iter()
            .filter(|import| {
                import.binding.is_none() && import.segments.last().is_some_and(|last| last == "*")
            })
            .find_map(|glob| {
                let source = &glob.segments[..glob.segments.len() - 1];
                match self.resolve(module, source, depth + 1) {
                    Resolution::Module(source) if source != module => {
                        self.resolve_name(source, name, depth + 1)
                    }
                    _ => None,
                }
            })
    }
}

impl Siblings {
    /// The nodes on a shortest path from `from` to `to`, both included.
    fn shortest_path(&self, from: Node, to: Node) -> Option<Vec<Node>> {
        let mut previous: BTreeMap<Node, Node> = BTreeMap::new();
        let mut queue = VecDeque::from([from]);
        let mut seen = BTreeSet::from([from]);
        while let Some(node) = queue.pop_front() {
            if node == to {
                let mut path = vec![to];
                let mut current = to;
                while let Some(before) = previous.get(&current) {
                    path.push(*before);
                    current = *before;
                }
                path.pop();
                path.reverse();
                path.insert(0, from);
                return Some(path);
            }
            for (_, next) in self.edges.keys().filter(|(start, _)| *start == node) {
                if seen.insert(*next) {
                    previous.insert(*next, node);
                    queue.push_back(*next);
                }
            }
        }
        None
    }
}

/// Walks one file, filling the module it is and the inline modules it holds.
struct Collector<'a> {
    crates: &'a mut Crates,
    /// Modules declared as `mod x;`, with the files that could hold them
    /// and whether each is a `mod.rs`.
    declared: Vec<(ModuleId, Vec<(PathBuf, bool)>)>,
    /// Where the children of each module on the stack live.
    dirs: Vec<PathBuf>,
    file: &'a SourceFile,
    stack: Vec<ModuleId>,
}

impl Collector<'_> {
    fn current(&mut self) -> &mut Module {
        let id = *self.stack.last().unwrap_or(&ModuleId(0));
        &mut self.crates.modules[id]
    }

    fn define(&mut self, ident: &syn::Ident) {
        let name = ident.to_string();
        self.current().defined.insert(name);
    }

    /// Whether the path at `span` carries a documented exception.
    fn is_allowed(&self, span: proc_macro2::Span) -> bool {
        let line = self.file.position_of(span).line;
        self.file.allowances.covers(Rule::ModuleCycle, line)
    }

    fn is_test(&self, span: proc_macro2::Span) -> bool {
        self.file.test_regions.contains(self.file.range(span).start)
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.define(&node.ident);
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.define(&node.ident);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.define(&node.sig.ident);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if let Some(ident) = &node.ident {
            self.define(ident);
        }
        syn::visit::visit_item_macro(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if self.is_test(node.span()) {
            return;
        }
        let parent = *self.stack.last().unwrap_or(&ModuleId(0));
        let name = node.ident.to_string();
        let id = ModuleId(self.crates.modules.len());
        let mut path = self.crates.modules[parent].path.clone();
        path.push(name.clone());
        let root = self.crates.modules[parent].root;
        self.crates.modules.push(Module {
            file: self.file.path.clone(),
            parent: Some(parent),
            path,
            root,
            ..Module::default()
        });
        self.crates.modules[parent].children.insert(name.clone(), id);
        let dir = self.dirs.last().cloned().unwrap_or_default();
        match &node.content {
            Some((_, items)) => {
                self.stack.push(id);
                self.dirs.push(dir.join(&name));
                for item in items {
                    self.visit_item(item);
                }
                self.dirs.pop();
                self.stack.pop();
            }
            None => {
                let explicit = node.attrs.iter().find_map(|attr| match &attr.meta {
                    syn::Meta::NameValue(pair) if pair.path.is_ident("path") => match &pair.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(value),
                            ..
                        }) => Some(value.value()),
                        _ => None,
                    },
                    _ => None,
                });
                let candidates = match explicit {
                    Some(relative) => {
                        let base = self.file.path.parent().unwrap_or(Path::new(""));
                        vec![(base.join(relative), true)]
                    }
                    None => vec![
                        (dir.join(format!("{name}.rs")), false),
                        (dir.join(&name).join("mod.rs"), true),
                    ],
                };
                self.declared.push((id, candidates));
            }
        }
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        self.define(&node.ident);
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.define(&node.ident);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.define(&node.ident);
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.define(&node.ident);
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_union(&mut self, node: &'ast syn::ItemUnion) {
        self.define(&node.ident);
        syn::visit::visit_item_union(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        if node.leading_colon.is_some() || self.is_test(node.span()) {
            return;
        }
        let exported = !matches!(node.vis, syn::Visibility::Inherited);
        let allowed = self.is_allowed(node.span());
        let offset = self.file.range(node.span()).start;
        let mut flat = Vec::new();
        flatten(&node.tree, &mut Vec::new(), &mut flat);
        for (segments, binding) in flat {
            self.current().imports.push(Import {
                allowed,
                binding,
                exported,
                offset,
                segments,
            });
        }
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(first) = node.segments.first() {
            let first = first.ident.to_string();
            self.current().idents.insert(first);
        }
        if node.leading_colon.is_none()
            && node.segments.len() > 1
            && !self.is_test(node.span())
            && !self.is_allowed(node.span())
        {
            let segments: Vec<String> = node
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            let offset = self.file.range(node.span()).start;
            self.current().references.push(Reference { offset, segments });
        }
        syn::visit::visit_path(self, node);
    }
}

/// The package directory that holds `path` under its `src/`, if any.
fn package_of(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .skip(1)
        .find(|dir| dir.join("Cargo.toml").is_file())
        .filter(|dir| path.starts_with(dir.join("src")))
        .map(Path::to_path_buf)
}

/// The crate roots of a package, library first.
fn crate_roots(package: &Path) -> Vec<PathBuf> {
    let src = package.join("src");
    let mut roots: Vec<PathBuf> = [src.join("lib.rs"), src.join("main.rs")]
        .into_iter()
        .filter(|path| path.is_file())
        .collect();
    if let Ok(entries) = std::fs::read_dir(src.join("bin")) {
        let mut bins: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter_map(|path| {
                if path.is_dir() {
                    Some(path.join("main.rs")).filter(|main| main.is_file())
                } else {
                    Some(path).filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
                }
            })
            .collect();
        bins.sort();
        roots.extend(bins);
    }
    roots
}

/// Every leaf of a use tree as a full path and the name it binds.
fn flatten(tree: &syn::UseTree, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, Option<String>)>) {
    match tree {
        syn::UseTree::Glob(_) => {
            let mut segments = prefix.clone();
            segments.push("*".to_string());
            out.push((segments, None));
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                flatten(item, prefix, out);
            }
        }
        syn::UseTree::Name(name) => {
            let ident = name.ident.to_string();
            if ident == "self" {
                let binding = prefix.last().cloned();
                out.push((prefix.clone(), binding));
            } else {
                let mut segments = prefix.clone();
                segments.push(ident.clone());
                out.push((segments, Some(ident)));
            }
        }
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            flatten(&path.tree, prefix, out);
            prefix.pop();
        }
        syn::UseTree::Rename(rename) => {
            let mut segments = prefix.clone();
            segments.push(rename.ident.to_string());
            let binding = Some(rename.rename.to_string()).filter(|name| name != "_");
            out.push((segments, binding));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A crate on disk from `(path, text)` pairs; returns its directory.
    fn krate(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rabot-cycles-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        for (relative, text) in [("Cargo.toml", "[package]\nname = \"x\"\n")].iter().chain(files) {
            let path = dir.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        dir
    }

    fn parse(path: PathBuf) -> SourceFile {
        let text = std::fs::read_to_string(&path).unwrap();
        SourceFile::parse(path, text).unwrap()
    }

    /// Lines reporting a cycle, per file relative to the crate, when
    /// `checked` files are the ones in scope.
    fn cycles(dir: &Path, checked: &[&str]) -> Vec<(String, usize)> {
        let files: Vec<SourceFile> = checked.iter().map(|path| parse(dir.join(path))).collect();
        let graph = ModuleGraph::build(&files);
        let mut found = Vec::new();
        for file in &files {
            for edge in graph.cycles_in(&file.path) {
                let relative = file.path.strip_prefix(dir).unwrap().display().to_string();
                found.push((relative, file.position(edge.offset).line));
            }
        }
        found
    }

    #[test]
    fn finds_a_cycle_through_the_crate_root_and_a_glob_reexport() {
        let dir = krate(
            "root",
            &[
                (
                    "src/lib.rs",
                    "mod ffi;\nmod worker;\npub use ffi::*;\nuse crate::worker::Worker;\npub type Inner = u8;\npub struct Client { worker: Worker }\n",
                ),
                ("src/ffi.rs", "pub mod error;\n"),
                ("src/ffi/error.rs", "pub struct ApiError;\n"),
                (
                    "src/worker.rs",
                    "use crate::{ApiError, Inner};\npub struct Worker { inner: Inner }\n",
                ),
            ],
        );
        let found = cycles(
            &dir,
            &["src/lib.rs", "src/ffi.rs", "src/ffi/error.rs", "src/worker.rs"],
        );
        assert_eq!(
            found,
            vec![("src/lib.rs".to_string(), 4), ("src/worker.rs".to_string(), 1)],
            "root <-> worker; `pub use ffi::*` is exposure, so worker -> ffi closes nothing"
        );
        // Only one side checked: the other is read from disk.
        assert_eq!(
            cycles(&dir, &["src/worker.rs"]),
            vec![("src/worker.rs".to_string(), 1)]
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn siblings_through_descendants_and_super() {
        let dir = krate(
            "siblings",
            &[
                ("src/lib.rs", "mod a;\nmod b;\n"),
                ("src/a/mod.rs", "mod deep;\n"),
                ("src/a/deep.rs", "pub fn f() { super::super::b::g(); }\n"),
                ("src/b.rs", "use crate::a::deep::f;\npub fn g() {}\n"),
            ],
        );
        let found = cycles(&dir, &["src/lib.rs", "src/a/mod.rs", "src/a/deep.rs", "src/b.rs"]);
        assert_eq!(
            found,
            vec![("src/a/deep.rs".to_string(), 1), ("src/b.rs".to_string(), 1)]
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn one_way_dependencies_tests_and_allowances_are_not_cycles() {
        let dir = krate(
            "clean",
            &[
                (
                    "src/lib.rs",
                    "mod a;\nmod b;\nmod c;\npub use a::A;\npub struct Root;\n",
                ),
                (
                    "src/a.rs",
                    "use crate::b::B;\nuse crate::Root;\npub struct A(B, Root);\n#[cfg(test)]\nmod tests { use crate::c::C; }\n",
                ),
                (
                    "src/b.rs",
                    "pub struct B;\n#[test]\nfn t() { let _ = crate::a::A; }\n",
                ),
                (
                    "src/c.rs",
                    "use crate::a::A;\n// rabot: allow(module-cycle) documented exception\nuse crate::b::B;\npub struct C;\n",
                ),
            ],
        );
        let all = ["src/lib.rs", "src/a.rs", "src/b.rs", "src/c.rs"];
        assert_eq!(cycles(&dir, &all), vec![]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_standalone_file_is_its_own_crate() {
        let dir = std::env::temp_dir().join(format!("rabot-cycles-single-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("single.rs");
        std::fs::write(
            &path,
            "mod a { use crate::b::B; pub struct A; }\nmod b { use crate::a::A; pub struct B; }\n",
        )
        .unwrap();
        let file = parse(path.clone());
        let graph = ModuleGraph::build(std::slice::from_ref(&file));
        assert_eq!(graph.cycles_in(&path).len(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }
}
