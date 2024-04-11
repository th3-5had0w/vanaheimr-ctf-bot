use teloxide::{prelude::*, utils::command::BotCommands};

use crate::arena_generator::{ARENA_PLAYER_IDS, HOSTIP, PLAYERS, PLAYER_01_PORT, PLAYER_02_PORT};
use crate::auth::verify;

pub async fn main_telegram_bot() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    Command::repl(bot.clone(), answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this help table")]
    Help,
    #[command(description = "join a ctf arena
        + usage: /join <id> <secret>", parse_with = "split")]
    Join {id: String, secret: String},
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Join {id, secret} => {
            let mut hash = String::new();
            let arena_state = verify(&id, &secret, &mut hash).await;
            match arena_state {
                0 => {
                    bot.send_message(
                        msg.chat.id,
                        format!("User {id} with secret {secret} is not available in the system.")
                    ).await?
                },
                1 => {
                    let mut players_guard = PLAYERS.lock().await;
                    players_guard.insert(hash, 1_u8);
                    let mut arena_player_ids_guard = ARENA_PLAYER_IDS.lock().await;
                    arena_player_ids_guard.push(msg.chat.id);
                    bot.send_message(
                        msg.chat.id,
                        format!("Your username is {id} verified with {secret}.")
                    ).await?;
                    if arena_player_ids_guard.len() == 2 {
                        let hostip = HOSTIP.lock().await;
                        bot.send_message(
                            *arena_player_ids_guard.first().expect("???"),
                            format!("Your opponent -> {}:{}", hostip, PLAYER_02_PORT)
                        ).await?;
                        bot.send_message(
                            *arena_player_ids_guard.last().expect("???"),
                            format!("Your opponent -> {}:{}", hostip, PLAYER_01_PORT)
                        ).await?
                    }
                    else {
                        bot.send_message(
                            msg.chat.id,
                            format!("Waiting for opponent...")
                        ).await?
                    }
                },
                2 => {
                    bot.send_message(
                        msg.chat.id,
                        format!("User {id} with secret {secret} has already joined.")
                    ).await?
                },
                3 => {
                    bot.send_message(
                        msg.chat.id,
                        format!("Room is full. Arena is now preparing for battle, can't join.")
                    ).await?
                },
                _ => {
                    bot.send_message(
                        msg.chat.id,
                        format!("What the fuck?")
                    ).await?
                }
            }
        }
    };
    Ok(())
}