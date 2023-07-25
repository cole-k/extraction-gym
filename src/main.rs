mod extract;

use clap::Parser;

pub use extract::*;

use egraph_serialize::*;

use indexmap::IndexMap;
use ordered_float::NotNan;

use anyhow::Context;

use std::io::Write;

pub type Cost = NotNan<f64>;
pub const INFINITY: Cost = unsafe { NotNan::new_unchecked(std::f64::INFINITY) };

#[derive(Parser, Debug)]
struct Args {
    /// Extractor to use
    #[arg(long = "extractor", default_value = "bottom-up")]
    extractor_name: String,

    /// Optional output filename
    #[arg(long = "out", default_value = "out.json")]
    out_filename: String,

    /// Input filename
    test_filename: String,
}

fn main() {
    env_logger::init();

    let extractors: IndexMap<&str, Box<dyn Extractor>> = [
        ("bottom-up", extract::bottom_up::BottomUpExtractor.boxed()),
        // (
        //     "greedy-dag",
        //     extract::greedy_dag::GreedyDagExtractor.boxed(),
        // ),
        #[cfg(feature = "ilp-cbc")]
        ("ilp-cbc", extract::ilp_cbc::CbcExtractor.boxed()),
        ("dijkstra", extract::dijkstra::DijkstraExtractor.boxed()),
    ]
    .into_iter()
    .collect();

    let args = Args::parse();

    if args.extractor_name == "print" {
        for name in extractors.keys() {
            println!("{}", name);
        }
        return;
    }

    let out_filename: String = args.out_filename.clone();

    let mut out_file = std::fs::File::create(out_filename).unwrap();

    let egraph = EGraph::from_json_file(&args.test_filename)
        .with_context(|| format!("Failed to parse {}", &args.test_filename))
        .unwrap();

    let class_parents = extract::compute_class_parents(&egraph);

    let extractor_name = args.extractor_name.clone();

    let extractor = extractors
        .get(extractor_name.as_str())
        .with_context(|| format!("Unknown extractor: {extractor_name}"))
        .unwrap();

    let start_time = std::time::Instant::now();
    let result = extractor.extract(&egraph, &egraph.root_eclasses, &class_parents);

    let us = start_time.elapsed().as_micros();
    assert!(result
        .find_cycles(&egraph, &egraph.root_eclasses)
        .is_empty());
    let tree = result.tree_cost(&egraph, &egraph.root_eclasses);
    let dag = result.dag_cost(&egraph, &egraph.root_eclasses);

    log::info!(
        "{:40}\t{extractor_name:10}\t{tree:5}\t{dag:5}\t{us:5}",
        &args.test_filename
    );
    writeln!(
        out_file,
        r#"{{ 
    "name": "{}",
    "extractor": "{extractor_name}", 
    "tree": {tree}, 
    "dag": {dag}, 
    "micros": {us}
}}"#,
        &args.test_filename
    )
    .unwrap();
}
