#[cfg(test)]
mod test_system {
    use super::super::*;
    use sysinfo::{System, SystemExt};

    #[test]
    fn test_get_logical_cpus() {
        let mut test_system = System::new_all();
        test_system.refresh_all();
        assert!(get_logical_cpus(&test_system) > 0);
    }

    #[test]
    fn test_get_hostname() {
        let mut test_system = System::new_all();
        test_system.refresh_all();
        assert!(get_hostname(&test_system).len() > 0);
    }
}
