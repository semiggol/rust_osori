use sysinfo::{ System, SystemExt, NetworkExt, ProcessorExt };

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

/// used cpu % since the last refresh
pub fn get_cpu_usage(s: &System) -> f32 {
    s.global_processor_info().cpu_usage()
}

/// hostname
pub fn get_hostname(s: &System) -> String {
    s.host_name().unwrap()
}

// logical cpu count
pub fn get_logical_cpus(s: &System) -> usize {
    s.processors().len()
}
