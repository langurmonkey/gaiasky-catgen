use crate::procinfo::pid;

pub fn log_mem() {
    let mem = pid::statm_self().expect("Error getting memory stats");
    log::info!(
        "MEM: total virt = {} KB, resident (non-swapped) = {} KB",
        mem.size,
        mem.resident
    );
}
