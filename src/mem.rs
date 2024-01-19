use procfs::process::Process;

pub fn log_mem() {
    let me = Process::myself().unwrap();
    let mem = me.stat().unwrap();
    let page_size = procfs::page_size();

    log::info!(
        "MEM: total pages = {:.3}, resident (non-swapped) = {:.3} MB",
        mem.rss,
        mem.rss as f64 * page_size as f64 / 1000.0
    );
}
