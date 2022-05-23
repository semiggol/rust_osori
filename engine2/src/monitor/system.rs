use std::io;
use cpu_monitor::CpuInstant;
use sysinfo::{ System, SystemExt, NetworkExt };

/// (used_memory, total_memory) KB
pub fn get_memory_usage(s: &System)-> (u64, u64) {
    let usage = s.used_memory();
    let total = s.total_memory();
    //println!("memory: {}/{} KB", usage, total);
    
    (usage, total)
}

/// (network input, output) KB since the last refresh
pub fn get_network_usage(s: &System)-> (u64, u64) {
    let mut network_in = 0;
    let mut network_out = 0;
    for (_, data) in s.networks() {
        //println!("{}: {}/{} B", interface_name, data.received(), data.transmitted()); 
        network_in += data.received();
        network_out += data.transmitted();
    }
    network_in /=1024;
    network_out /=1024;
    //println!("network: {}/{} KB", network_in, network_out);

    (network_in, network_out)
}

/// (end CpuInstant, used_cpu) KB
pub fn get_cpu_usage(start: CpuInstant) -> Result<(CpuInstant, f64), io::Error> {
    let end = CpuInstant::now()?;
    let duration = end - start;
    let usage = duration.non_idle() * 100.;
    //println!("cpu: {:.0}%", usage);

    Ok((end, usage))
}

/// hostname
pub fn get_hostname(s: &System) -> String {
    s.host_name().unwrap()
}

// logical cpu count
pub fn get_logical_cpus() -> usize {
    num_cpus::get()
}
