use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

use vigem_client_c::Client;

#[test]
fn test_drop() {
    struct DropChecker<'flag> {
        flag: &'flag AtomicBool,
    }

    impl Drop for DropChecker<'_> {
        fn drop(&mut self) {
            self.flag.store(true, SeqCst);
        }
    }

    let client = Client::new().unwrap();
    let mut pad = client.connect_x360_pad().unwrap();
    let flag = AtomicBool::new(false);
    let _checker = DropChecker { flag: &flag };

    let handle = pad
        .register_notification(move |_| {
            let _checker = &_checker;
        })
        .unwrap();
    assert!(!flag.load(SeqCst));
    pad.unregister_notification(handle);
    assert!(flag.load(SeqCst));
}
