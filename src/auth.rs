use std::{cmp::Ordering, collections::HashMap};

use lazy_static::lazy_static;
use tokio::{io::AsyncReadExt, sync::{Mutex, MutexGuard}};
use sha2::{Sha224, Digest};

use crate::arena_generator::PLAYERS;

lazy_static!{
    static ref DB: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub async fn read_db() {
    let mut db = DB.lock().await;
    let mut dbfile = tokio::fs::File::open("db.txt").await.expect("error opening db file");
    let mut tmp_buf: Vec<u8> = Vec::new();
    let _ = dbfile.read_to_end(&mut tmp_buf).await;
    let mut iter = tmp_buf.split(|delim| {
        *delim == b'\n' || *delim == b'\t' || *delim == b' '
    });
    let mut val = iter.next();
    while val.is_some() {
        let identity = String::from_utf8(val.unwrap().to_vec()).expect("invalid db data");
        db.push(identity.to_ascii_lowercase());
        val = iter.next();
    }
}

async fn is_player_joined(players_guard: &MutexGuard<'static, HashMap<String, u8>>, player_hash: &String) -> bool {
    players_guard.contains_key(player_hash)
}

async fn is_room_full(players_guard: &MutexGuard<'static, HashMap<String, u8>>) -> bool {
    players_guard.len() > 2
}

async fn is_player_valid(hash_res: &mut String, search_data : &String) -> u8 {
    let db = DB.lock().await;
    if let Ok(_exist) = db.binary_search_by(|probe| {
        let mut wrong = false;
        
        let a1 = search_data.as_bytes();
        let a2 = probe.as_bytes();
        for cnt in 0..56 {
            if a1[cnt] != a2[cnt] {
                wrong = true;
                break;
            }
        }

        if wrong {
            Ordering::Greater
        }
        else {
            Ordering::Equal
        }
    }) {
        hash_res.clone_from(&search_data);
        1
    }
    else {
        0
    }
}

pub async fn verify(id: &String, secret: &String, hash_res: &mut String) -> u8 {
    let players_guard = PLAYERS.lock().await;


    if is_room_full(&players_guard).await {
        return 3
    }

    let mut id_secret = secret.clone();
    id_secret.insert_str(0, &id);
    let hash = Sha224::digest(id_secret);
    let search_data = format!("{:02x}", &hash);
    println!("{}", search_data);
    
    if is_player_joined(&players_guard, &search_data).await {
        return 2
    }

    is_player_valid(hash_res, &search_data).await
}