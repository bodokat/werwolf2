use std::iter::once;

use crate::{
    lobby::{PlayerMessage, Settings},
    message,
};
use futures::{
    future::ready,
    stream::{FuturesOrdered, FuturesUnordered, StreamExt},
};
use itertools::{repeat_n, Itertools};
use rand::prelude::*;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::roles::{Group, Role, RoleBehavior, Team};

pub async fn start(players: Vec<(&String, UnboundedSender<PlayerMessage>)>, settings: &Settings) {
    let (mut data, behaviors) = setup(players, settings);

    data.players
        .iter()
        .zip(data.roles.iter())
        .for_each(|(c, role)| c.say(format!("Deine Rolle ist: {}", role.name())));

    data.players
        .iter()
        .for_each(|c| c.say("Die Abstimmung beginnt".into()));

    // --- Action

    actions(behaviors, &mut data).await;

    // --- Voting

    data.players
        .iter()
        .for_each(|c| c.say("Die Abstimmung beginnt".into()));

    let votes = do_vote(&data).await;

    {
        let content = format!(
            "Stimmen:\n{}",
            votes
                .iter()
                .enumerate()
                .map(|(idx, &vote)| format!(
                    "{}: {}",
                    data.players[idx].name,
                    match vote {
                        Some(v) => data.players[v].name,
                        None => "Skip",
                    }
                ))
                .join("\n")
        );
        data.players.iter().for_each(|p| p.say(content.clone()));
    }

    let killed_players = killed_players(&votes);

    {
        let content = if killed_players.is_empty() {
            "Niemand ist gestorben".to_string()
        } else if killed_players.len() == 1 {
            format!(
                "{} ist gestorben",
                data.players[killed_players[0]].name.clone()
            )
        } else {
            format!(
                "{} sind gestorben",
                killed_players
                    .iter()
                    .map(|&idx| data.players[idx].name.clone())
                    .join(", ")
            )
        };
        data.players.iter().for_each(|p| p.say(content.clone()));
    }

    let has_werewolf = data.roles.iter().any(|role| role.group() == Group::Wolf);

    #[allow(clippy::collapsible_else_if)]
    let winning_team = if has_werewolf {
        if killed_players
            .iter()
            .any(|&idx| data.roles[idx].group() == Group::Wolf)
        {
            Team::Dorf
        } else {
            Team::Wolf
        }
    } else {
        if killed_players.is_empty() {
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

fn killed_players(votes: &[Option<usize>]) -> Vec<usize> {
    let total_votes = votes.iter().filter_map(Option::as_ref).fold(
        vec![0_usize; votes.len()],
        |mut votes, &v| {
            votes[v] += 1;
            votes
        },
    );
    let Some(max_votes) = total_votes.iter().max() 
    else {
        return Vec::new();
    };
    let skipped = votes.iter().filter(|x| x.is_none()).count();
    if skipped > *max_votes {
        return Vec::new();
    }
    total_votes
        .iter()
        .enumerate()
        .filter(|&(_i, v)| v == max_votes)
        .map(|(i, _v)| i)
        .collect_vec()
}

async fn do_vote(data: &Data<'_>) -> Vec<Option<usize>> {
    let options = (0..data.players.len())
        .map(Some)
        .chain(once(None))
        .collect::<Vec<_>>();
    let options_str = options
        .iter()
        .map(|&x| match x {
            Some(i) => data.players[i].name.clone(),
            None => "Skip".into(),
        })
        .collect::<Vec<_>>();

    let options_str = &options_str;

    data.players
        .iter()
        .map(|player| player.choice("Wähle einen Mitspieler".into(), options_str.clone()))
        .collect::<FuturesOrdered<_>>()
        .map(|v| options[v])
        .collect::<Vec<_>>()
        .await
}

async fn actions(mut behaviors: Vec<Box<dyn RoleBehavior>>, data: &mut Data<'_>) {
    behaviors
        .iter_mut()
        .enumerate()
        .map(|(idx, behavior)| behavior.before_ask(&*data, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    behaviors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, behavior)| behavior.before_action(data, idx));

    behaviors
        .iter_mut()
        .enumerate()
        .map(|(idx, behavior)| behavior.ask(&*data, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    behaviors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, behavior)| behavior.action(data, idx));

    behaviors
        .iter_mut()
        .enumerate()
        .map(|(idx, behavior)| behavior.after(&*data, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;
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
