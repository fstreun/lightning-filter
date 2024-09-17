#[macro_use] extern crate nickel;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use nickel::{Nickel, HttpRouter};
use serde_json::Value;
use clap::Parser;

mod prometheus;
mod collector;
mod dpdk;

use prometheus::PrometheusMetrics;
use collector::Collector;
use dpdk::DpdkTelemetry;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address and port to expose the metrics
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    address: String,

    /// Path to expose the metrics
    #[arg(short, long, default_value = "/metrics")]
    path: String,

    /// DPDK file prefix for determine DPDK runtime directory
    #[arg(short, long)]
    dpdk_file_prefix: String,
}

fn main() {

    let args = Args::parse();

    // Create DPDK Telemetry object
    let dpdk_runtime_dir = get_dpdk_runtime_dir(&args.dpdk_file_prefix);
    let dpdk_telemetry: DpdkTelemetry = DpdkTelemetry::new(dpdk_runtime_dir);
    // Some example queries
    let response: Value = dpdk_telemetry.query("/");
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    let response = dpdk_telemetry.query("/ethdev/list");
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    let response = dpdk_telemetry.query("/ethdev/stats");
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    let response = dpdk_telemetry.query("/lf/keymanager/worker/stats,1");
    println!("{}", serde_json::to_string_pretty(&response).unwrap());

    // Create Prometheus metrics
    let prometheus_metrics: PrometheusMetrics = PrometheusMetrics::new();
    let registry: Registry = prometheus::new_registry(&prometheus_metrics);

    // Create metric collector and start it
    let collector: Collector = Collector::new(dpdk_telemetry, prometheus_metrics);
    std::thread::spawn(move || collector.run());

    // Start HTTP server
    let mut server: Nickel = Nickel::new();
    server.get(args.path, middleware! {
        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();
        buffer
    });
    server.listen(args.address).unwrap();

}


fn get_dpdk_runtime_dir(dpdk_file_prefix: &str) -> String {
    let dpdk_runtime_dir = format!("/var/run/dpdk/{}", dpdk_file_prefix);
    return dpdk_runtime_dir;
}