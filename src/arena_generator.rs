use std::{collections::HashMap, io::Write};
use bollard::{container::{Config, CreateContainerOptions, InspectContainerOptions, StartContainerOptions}, image::BuildImageOptions, models::{HostConfig, PortBinding}, network::{CreateNetworkOptions, InspectNetworkOptions}, Docker};
use futures_util::stream::StreamExt;
use tokio::{io::AsyncReadExt, sync::Mutex};
use teloxide::types::ChatId;
use lazy_static::lazy_static;
use get_if_addrs::get_if_addrs;

use crate::network::arena_network_interface;

use uuid::Uuid;

const IMAGE_01_BUILD_NAME: &'static str = "battlefield:c1";
const IMAGE_02_BUILD_NAME: &'static str = "battlefield:c2";
const PLAYER_01_ID_CONTAINER_NAME: &'static str = "usa";
const PLAYER_02_ID_CONTAINER_NAME: &'static str = "ussr";
const DEFAULT_NETWORK_NAME: &'static str = "cold_war";
const DEFAULT_REDJAIL_PORT_CONFIG: &'static str = "5000/tcp";
pub const PLAYER_01_PORT: u16 = 50001;
pub const PLAYER_02_PORT: u16 = 40001;

lazy_static! {
    pub static ref PLAYERS: Mutex<HashMap<String, u8>> = Mutex::new(HashMap::new());
    pub static ref ARENA_PLAYER_IDS: Mutex<Vec<ChatId>> = Mutex::new(Vec::new());
    pub static ref HOSTIP: Mutex<String> = Mutex::new(String::new());
}

async fn create_arena_network(docker_handler : &Docker) {

    let network_options = CreateNetworkOptions {
        name: DEFAULT_NETWORK_NAME,
        check_duplicate: true,
        ..CreateNetworkOptions::default()
    };

    if let Err(_) = docker_handler.create_network(network_options).await {
        println!("network existed");
    }
}


async fn prepare_docker_image(docker_handler : &Docker, name_and_tag: &str) {
    let mut dockerfile_file_handle = tokio::fs::File::open("Dockerfile").await.expect("cannot open dockerfile");
    let mut content_vec = Vec::new();
    dockerfile_file_handle.read_to_end(&mut content_vec).await.expect("failed reading dockerfile");
    let dockerfile_content_str = String::from_utf8(content_vec).expect("invalid characters in dockerfile");
    let dockerfile_content = dockerfile_content_str.replace("<FLAG>", &Uuid::new_v4().to_string());
    let dockerfile = format!("{}", dockerfile_content);

    let mut header = tar::Header::new_gnu();
    header.set_path("Dockerfile").unwrap();
    header.set_size(dockerfile.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(&header, dockerfile.as_bytes()).unwrap();

    let uncompressed = tar.into_inner().unwrap();
    let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    c.write_all(&uncompressed).unwrap();
    let compressed = c.finish().unwrap();

    let build_options = BuildImageOptions {
        dockerfile: "Dockerfile",
        t: name_and_tag,
        pull: true,
        rm: true,
        ..BuildImageOptions::default()
    };

    let mut vkl = docker_handler.build_image(
        build_options, 
        None, 
        Some(compressed.into()));
    
    while let Some(msg) = vkl.next().await {
        println!("Message: {:?}", msg);
    }
}

async fn prepare_docker_container(docker_handler : &Docker, container_name: &str, image_name: &str, host_expose_port: u16) {
    let create_options = Some(CreateContainerOptions {
        name: container_name,
        platform: None
    });

    let mut port_bindings = ::std::collections::HashMap::new();
    port_bindings.insert(
        String::from(DEFAULT_REDJAIL_PORT_CONFIG),
        Some(vec![PortBinding {
            host_ip: Some(String::from("0.0.0.0")),
            host_port: Some(host_expose_port.to_string()),
        }]),
    );

    let host_config = HostConfig {
        privileged: Some(true),
        network_mode: Some(DEFAULT_NETWORK_NAME.to_string()),
        port_bindings: Some(port_bindings),
        ..HostConfig::default()
    };

    let mut exposed_ports = HashMap::new();
    exposed_ports.insert(DEFAULT_REDJAIL_PORT_CONFIG, HashMap::new());

    let config = Config {
        image: Some(image_name),
        host_config: Some(host_config),
        exposed_ports: Some(exposed_ports),
        ..Default::default()
    };

    let create_container_result = docker_handler.create_container(create_options, config).await.expect("failed creating docker container");
    docker_handler.start_container(&create_container_result.id, None::<StartContainerOptions<String>>).await.expect("failed starting docker container");
}

async fn create_arena(docker_handler : &Docker) {
    prepare_docker_image(&docker_handler, IMAGE_01_BUILD_NAME).await;
    prepare_docker_image(&docker_handler, IMAGE_02_BUILD_NAME).await;
    prepare_docker_container(&docker_handler, PLAYER_01_ID_CONTAINER_NAME, IMAGE_01_BUILD_NAME, PLAYER_01_PORT).await;
    prepare_docker_container(&docker_handler, PLAYER_02_ID_CONTAINER_NAME, IMAGE_02_BUILD_NAME, PLAYER_02_PORT).await;
}

async fn submit_container_info(docker_handler : &Docker, container_name: &str) {
    let vkl = docker_handler.inspect_container(container_name, Some(InspectContainerOptions {size: false})).await.expect("failed to get container info");
    let a = vkl.network_settings.unwrap();
}

async fn public_network_info(docker_handler : &Docker, network_name: &str) {
    let options = InspectNetworkOptions {
        verbose: true,
        scope: "local",
    };
    let docker_network_info = docker_handler.inspect_network(network_name, Some(options)).await.expect("failed getting network info");
    let mut network_interface_guard = arena_network_interface.lock().await;
    network_interface_guard.clone_from(&format!("br-{}", docker_network_info.id.unwrap().get(0..12).unwrap()));
}

async fn set_lan_eth0_ip() {
    let ifconf = get_if_addrs().expect("failed getting ifconfig");
    for interface in ifconf {
        if interface.name.eq_ignore_ascii_case("eth0") {
            HOSTIP.lock().await.insert_str(0, &interface.addr.ip().to_string());
        }
    }
}

pub async fn main_arena_generator() {
    let docker_handler = Docker::connect_with_local_defaults().expect("failed to connect to docker service");
    set_lan_eth0_ip().await;
    create_arena_network(&docker_handler).await;
    create_arena(&docker_handler).await;
}