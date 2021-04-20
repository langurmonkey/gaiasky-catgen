use crate::procinfo::pid;

pub fn log_mem() {
    let mem = pid::statm_self().expect("Error getting memory stats");
    log::info!(
        "MEM: total virt = {:.3} MB, resident (non-swapped) = {:.3} MB",
        mem.size as f64 / 1000.0,
        mem.resident as f64 / 1000.0
    );
}
