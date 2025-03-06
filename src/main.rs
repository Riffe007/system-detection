mod backend;
use backend::system_monitor::SystemSpecs;

fn main() {
    println!("🔍 Detecting system hardware...");
    let system_specs = SystemSpecs::detect();
    
    println!("🛠 Recommended Configuration:");
    let config = system_specs.recommend_config();
    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    println!("💾 Saving configuration to `config.json`...");
    system_specs.save_config();

    println!("✅ System detection complete!");
}
