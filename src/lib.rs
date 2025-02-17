use std::path::{Path, PathBuf};

use glob::Pattern;
use graphviz_rust::{
    cmd::Format, dot_structures::{Attribute, Edge, EdgeTy, Graph, Id, Node, NodeId, Stmt, Vertex}, printer::{DotPrinter, PrinterContext}
};
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};
use oxc_resolver::{ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};
use petgraph::prelude::StableGraph;

pub mod app;
pub mod js;

#[derive(Clone)]
pub struct ScannerOptions {
    pub filter: Vec<Pattern>,
    pub include: Option<Vec<Pattern>>,
}

impl Default for ScannerOptions {
    fn default() -> Self {
        Self {
            filter: vec![Pattern::new("*node_modules/*").unwrap()],
            include: None,
        }
    }
}

pub struct Scanner {
    allocator: Allocator,
    resolver: Resolver,
    options: ScannerOptions,
}

#[derive(Debug)]
pub struct Container {
    nodes: Vec<PathBuf>,
    edges: Vec<(PathBuf, PathBuf)>,
    include: Option<Vec<Pattern>>,
}

impl Container {
    pub fn new(include: Option<Vec<Pattern>>) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            include,
        }
    }

    pub fn build_petgraph(&self) -> StableGraph<String, ()> {
        let mut g = StableGraph::new();
        
        for node in self.nodes.iter().filter(|n| match &self.include {
            Some(includes) => includes.iter().any(|i| i.matches_path(n)),
            None => true,
        }) {
            g.add_node(node.to_string_lossy().to_string());
        }

        for (source, target) in self
            .edges
            .iter()
            .filter(|(source, target)| match &self.include {
                Some(includes) => {
                    includes.iter().any(|i| i.matches_path(source))
                        && includes.iter().any(|i| i.matches_path(target))
                }
                None => true,
            })
        {
            let source = g.node_indices().find(|i| g[*i] == source.to_string_lossy().to_string()).unwrap();
            let target = g.node_indices().find(|i| g[*i] == target.to_string_lossy().to_string()).unwrap();
            g.add_edge(source, target, ());
        }

        g
    }

    fn build_graph(&self) -> Graph {
        let mut graph = Graph::DiGraph {
            id: Id::Plain("main".to_string()),
            strict: false,
            stmts: vec![],
        };

        for node in self.nodes.iter().filter(|n| match &self.include {
            Some(includes) => includes.iter().any(|i| i.matches_path(n)),
            None => true,
        }) {
            graph.add_stmt(Stmt::Node(Node::new(
                path_to_node_id(node),
                vec![Attribute(
                    Id::Plain("shape".to_string()),
                    Id::Plain("record".to_string()),
                )],
            )));
        }

        for (source, target) in self
            .edges
            .iter()
            .filter(|(source, target)| match &self.include {
                Some(includes) => {
                    includes.iter().any(|i| i.matches_path(source))
                        && includes.iter().any(|i| i.matches_path(target))
                }
                None => true,
            })
        {
            graph.add_stmt(Stmt::Edge(Edge {
                ty: EdgeTy::Pair(
                    Vertex::N(path_to_node_id(source)),
                    Vertex::N(path_to_node_id(target)),
                ),
                attributes: vec![],
            }));
        }

        graph
    }

    pub fn into_svg(&self) -> Result<Vec<u8>, std::io::Error> {
        let graph = self.build_graph();
        graphviz_rust::exec(graph, &mut PrinterContext::default(), vec![Format::Svg.into()])
    }

    pub fn print_graphviz(&self) -> String {
        let graph = self.build_graph();
        return graph.print(&mut PrinterContext::default());
    }
}

fn path_to_node_id(path: &PathBuf) -> NodeId {
    NodeId(
        Id::Escaped(format!(
            "\"{}\"",
            path.to_str().expect("Failed to parse").to_string()
        )),
        None,
    )
}

impl Scanner {
    pub fn new(root: PathBuf, options: ScannerOptions) -> Self {
        Self {
            allocator: Allocator::new(),
            resolver: Resolver::new(ResolveOptions {
                tsconfig: Some(TsconfigOptions {
                    config_file: Path::new(format!("{}/tsconfig.json", root.display()).as_str())
                        .to_path_buf(),
                    references: TsconfigReferences::Auto,
                }),
                extensions: vec![
                    ".ts".to_string(),
                    ".js".to_string(),
                    ".json".to_string(),
                    ".node".to_string(),
                ],
                prefer_relative: true,
                ..ResolveOptions::default()
            }),
            options,
        }
    }

    pub fn set_filters(&mut self, filters: Vec<Pattern>) {
        self.options.filter = filters;
    }

    pub fn set_includes(&mut self, includes: Option<Vec<Pattern>>) {
        self.options.include = includes;
    }

    pub fn scan(&self, path: &PathBuf) -> Container {
        let mut graph = Container::new(self.options.include.clone());
        self.next(&mut graph, path);
        graph
    }

    pub fn next(&self, graph: &mut Container, path: &PathBuf) {
        let cwd = path.parent().unwrap();
        let source_text = std::fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap();
        let parsed = Parser::new(&self.allocator, &source_text, source_type).parse();
        graph.nodes.push(path.to_owned());

        parsed
            .module_record
            .requested_modules
            .into_iter()
            .filter_map(
                |(mod_name, _)| match self.resolver.resolve(cwd, &mod_name) {
                    Ok(resolved) => Some(resolved.full_path()),
                    Err(_) => None,
                },
            )
            .filter(|path| !self.options.filter.iter().any(|p| p.matches_path(path)))
            .for_each(|import_path| {
                graph.edges.push((path.to_owned(), import_path.to_owned()));
                if graph.nodes.contains(&import_path) {
                    return;
                }

                self.next(graph, &import_path);
            });
    }
}
