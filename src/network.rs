use lazy_static::lazy_static;
use pcap::Device;
use tokio::sync::Mutex;

lazy_static! {
    pub static ref arena_network_interface: Mutex<String> = Mutex::new(String::new());
}

async fn get_arena_device() -> Result<Device, &'static str> {
    let arena_network_interface_guard = arena_network_interface.lock().await;
    let device_list = Device::list().expect("failed getting device list");
    for device in device_list {
        if device.name.eq_ignore_ascii_case(&arena_network_interface_guard.to_ascii_lowercase()) {
            return Ok(device)
        }
    }
    Err("failed getting arena device")
}

pub async fn intercepting_arena() {
    let arena_device = get_arena_device().await.expect("msg");
    println!("{}", arena_device.name);
    let mut arena_device_handle = arena_device
                                                                            .open().expect("failed opening arena device");
                                                                            //.setnonblock().expect("failed setting device handle nonblock");
    let vkl = arena_device_handle.next_packet().expect("msg");
    vkl.to_vec();
}

pub async fn get_container_network_info() {

}

fn network_main() {
    
}