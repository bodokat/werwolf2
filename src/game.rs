use crate::{
    lobby::{PlayerMessage, Settings},
    message,
};
use futures::{
    future::ready,
    stream::{FuturesUnordered, StreamExt},
};
use itertools::{repeat_n, Itertools};
use rand::prelude::*;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::roles::{Group, Role, RoleBehavior, Team};

pub async fn start(players: Vec<(&String, UnboundedSender<PlayerMessage>)>, settings: &Settings) {
    let (mut data, mut behaviors) = setup(players, settings);

    let roles_string = data.roles.iter().join(" | ");

    data.players
        .iter()
        .for_each(|c| c.say(format!("Die Rollen sind:\n{roles_string}")));

    data.players
        .iter()
        .zip(data.roles.iter())
        .for_each(|(c, role)| c.say(format!("Deine Rolle ist: {}", role.name())));

    // --- Action

    behaviors
        .iter_mut()
        .enumerate()
        .map(|(idx, behavior)| behavior.before_ask(&data, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    behaviors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, behavior)| behavior.before_action(&mut data, idx));

    behaviors
        .iter_mut()
        .enumerate()
        .map(|(idx, behavior)| behavior.ask(&data, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    behaviors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, behavior)| behavior.action(&mut data, idx));

    behaviors
        .iter_mut()
        .enumerate()
        .map(|(idx, behavior)| behavior.after(&data, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    // --- Voting

    data.players
        .iter()
        .for_each(|c| c.say("Die Abstimmung beginnt".into()));

    let players = &data.players;
    let votes: Vec<u32> = data
        .players
        .iter()
        .map(|player| {
            player.choice(
                "Wähle einen Mitspieler".into(),
                (0..players.len())
                    .map(|idx| players[idx].name.clone())
                    .collect(),
            )
        })
        .collect::<FuturesUnordered<_>>()
        .fold(vec![0; data.roles.len()], |mut acc, x| {
            acc[x] += 1;
            ready(acc)
        })
        .await;

    {
        let content = format!(
            "Stimmen:\n{}",
            votes
                .iter()
                .enumerate()
                .map(|(idx, votes)| format!("{}: {}", data.players[idx].name, votes))
                .join("\n")
        );
        data.players.iter().for_each(|p| p.say(content.clone()));
    }

    let mut skipped = false;
    let voted_players =
        votes
            .iter()
            .enumerate()
            .fold(
                (Vec::new(), 1_u32),
                |(mut result, max), (idx, &votes)| match votes.cmp(&max) {
                    std::cmp::Ordering::Less => {
                        skipped = true;
                        (result, max)
                    }
                    std::cmp::Ordering::Equal => {
                        result.push(idx);
                        (result, max)
                    }
                    std::cmp::Ordering::Greater => {
                        skipped = true;
                        (vec![idx], votes)
                    }
                },
            );
    let voted_players = if skipped { voted_players.0 } else { vec![] };

    {
        let content = if voted_players.is_empty() {
            "Niemand ist gestorben".to_string()
        } else if voted_players.len() == 1 {
            format!(
                "{} ist gestorben",
                data.players[voted_players[0]].name.clone()
            )
        } else {
            format!(
                "{} sind gestorben",
                voted_players
                    .iter()
                    .map(|&idx| data.players[idx].name.clone())
                    .join(", ")
            )
        };
        let content = &content;
        data.players.iter().for_each(|p| p.say(content.to_string()));
    }

    let has_werewolf = data.roles.iter().any(|role| role.group() == Group::Wolf);

    #[allow(clippy::collapsible_else_if)]
    let winning_team = if has_werewolf {
        if voted_players
            .iter()
            .any(|&idx| data.roles[idx].group() == Group::Wolf)
        {
            Team::Dorf
        } else {
            Team::Wolf
        }
    } else {
        if voted_players.is_empty() {
            Team::Dorf
        } else {
            Team::Wolf
        }
    };

    {
        let content = match winning_team {
            Team::Dorf => "Das Dorf hat gewonnen",
            Team::Wolf => "Die Werwölfe haben gewonnen",
        };
        data.players.iter().for_each(|p| p.say(content.into()));
    }

    {
        let winners = data
            .players
            .iter()
            .zip(data.roles.iter())
            .filter_map(|(player, role)| {
                if role.team() == winning_team {
                    Some(player.name.clone())
                } else {
                    None
                }
            })
            .join(", ");

        let content = &format!("Die Gewinner sind: {winners}");
        data.players.iter().for_each(|p| p.say(content.to_string()));
    }

    data.players
        .iter()
        .for_each(|p| p.say("-------------".into()));
}

pub struct Data<'a> {
    pub players: Vec<Player<'a>>,
    pub roles: Vec<&'static dyn Role>,
    pub extra_roles: Vec<&'static dyn Role>,
}

pub struct Player<'a> {
    pub name: &'a String,
    sender: UnboundedSender<PlayerMessage>,
}

fn setup<'a>(
    players: Vec<(&'a String, UnboundedSender<PlayerMessage>)>,
    settings: &'a Settings,
) -> (Data<'a>, Vec<Box<dyn RoleBehavior>>) {
    let mut all_roles = settings
        .role_amounts
        .iter()
        .enumerate()
        .flat_map(|(index, &amount)| repeat_n(settings.available_roles[index], amount))
        .collect::<Vec<_>>();

    assert!(players.len() <= all_roles.len(), "Not enough roles");

    let mut thread_rng = thread_rng();
    all_roles.shuffle(&mut thread_rng);
    let (roles, extra_roles) = all_roles.split_at(players.len());
    let roles = roles.to_vec();
    let extra_roles = extra_roles.to_vec();

    let behaviors = roles.iter().map(|r| r.build()).collect::<Vec<_>>();

    let players = players
        .into_iter()
        .map(|(name, sender)| Player { name, sender })
        .collect();

    (
        Data {
            players,
            roles,
            extra_roles,
        },
        behaviors,
    )
}

impl<'a> Player<'a> {
    pub fn say(&self, text: String) {
        self.sender
            .send(PlayerMessage::Other(message::ToClient::Text { text }))
            .unwrap();
    }
    pub async fn choice(&self, text: String, options: Vec<String>) -> usize {
        assert!(!options.is_empty());
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(PlayerMessage::Question {
                text,
                options,
                response: tx,
            })
            .unwrap();
        rx.await.unwrap()
    }
}
