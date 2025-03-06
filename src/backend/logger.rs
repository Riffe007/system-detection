use flexi_logger::Logger;

pub fn init_logger() {
    Logger::try_with_str("info")
        .unwrap()
        .start()
        .unwrap();
}
