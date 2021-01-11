use std::collections::{HashMap, HashSet};

use futures::{future::select, pin_mut};
use serenity::{client::Context, model::user::User};
use tokio::sync::mpsc;

use crate::{controller::ReactionAction, game::start_game};

pub struct Lobby(pub mpsc::Sender<Message>);

pub enum Message {
    Reaction(ReactionAction),
    Join(User),
    Leave(User),
    Start,
}

#[derive(Default)]
struct LobbyData {
    players: HashSet<User>,
}

impl Lobby {
    pub fn new(ctx: Context) -> Self {
        let data = LobbyData::default();

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(lobby_loop(ctx, data, rx));

        Lobby(tx)
    }
}

async fn lobby_loop(ctx: Context, mut data: LobbyData, mut rc: mpsc::Receiver<Message>) {
    'outer: while let Some(msg) = rc.recv().await {
        match msg {
            Message::Reaction(_) => (),
            Message::Join(u) => {
                data.players.insert(u);
            }
            Message::Leave(u) => {
                data.players.remove(&u);
                if data.players.is_empty() {
                    println!("Deleting Lobby (no more players)");
                    break;
                }
            }
            Message::Start => {
                let mut senders = HashMap::with_capacity(data.players.len());
                let mut recievers = HashMap::with_capacity(data.players.len());

                for user in data.players.iter() {
                    let (tx, rx) = mpsc::channel(32);
                    senders.insert(user.id, tx);
                    recievers.insert(user.id, rx);
                }

                let game = start_game(&ctx, &data.players, recievers);
                pin_mut!(game);
                loop {
                    let rec = rc.recv();
                    pin_mut!(rec);
                    match select(&mut game, rec).await {
                        futures::future::Either::Left(_) => {
                            break;
                        }
                        futures::future::Either::Right((Some(Message::Reaction(r)), _)) => {
                            if let Some(sender) = r.inner().user_id.and_then(|id| senders.get(&id))
                            {
                                let _ = sender.try_send(r);
                            }
                        }
                        futures::future::Either::Right((None, _)) => {
                            break 'outer;
                        }
                        futures::future::Either::Right(_) => (),
                    }
                }
            }
        }
    }
}
