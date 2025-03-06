mod backend;
use backend::system_monitor::SystemMonitor;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let metrics = rt.block_on(SystemMonitor::collect_metrics());

    for (key, value) in metrics.iter() {
        println!("{}: {}", key, value);
    }
}
