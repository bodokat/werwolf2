use futures::{
    future::{join_all, ready},
    stream::FuturesOrdered,
};
use itertools::Itertools;
use rand::prelude::*;
use serenity::{
    framework::standard::CommandResult,
    futures::stream::{FuturesUnordered, StreamExt},
    model::{
        channel::{PrivateChannel, ReactionType},
        id::ChannelId,
        prelude::User,
    },
    prelude::*,
};
use std::collections::HashMap;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    controller::ReactionAction,
    roles::{self, Group, Role, RoleBehavior, Team},
};

pub async fn start_game(
    ctx: &Context,
    players: Vec<(&User, ReceiverStream<ReactionAction>)>,
) -> CommandResult {
    let roles: Vec<Box<dyn Role>> = vec![
        Box::new(roles::Doppel),
        Box::new(roles::Werwolf),
        Box::new(roles::Werwolf),
        Box::new(roles::Dorfbewohner),
        Box::new(roles::Seherin),
        Box::new(roles::Dieb),
        Box::new(roles::Unruhestifterin),
        Box::new(roles::Schlaflose),
    ];

    let roles_string = roles.iter().join(" | ");

    join_all(players.iter().map(|(player, _)| {
        player.dm(ctx, |m| {
            m.content(format!("Die Rollen sind:\n{}", roles_string))
        })
    }))
    .await;

    let (mut data, mut behaviors, mut reactions) = setup(players, roles, ctx).await;

    join_all(data.users.iter().zip(data.roles.iter()).map(|(u, role)| {
        u.dm(ctx, move |m| {
            m.content(format!("Deine Rolle ist: {}", role))
        })
    }))
    .await;

    // --- Action

    behaviors
        .iter_mut()
        .zip(reactions.iter_mut())
        .enumerate()
        .map(|(idx, (behavior, reactions))| behavior.ask(&data, reactions, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    behaviors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, behavior)| behavior.action(&mut data, idx));

    behaviors
        .iter_mut()
        .zip(reactions.iter_mut())
        .enumerate()
        .map(|(idx, (behavior, reactions))| behavior.after(&data, reactions, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    // --- Voting

    join_all(
        data.users
            .iter()
            .map(|u| u.dm(ctx, |m| m.content("Voting started"))),
    )
    .await;

    let users = &data.users;
    let votes: Vec<u32> = reactions
        .iter_mut()
        .zip(data.dm_channels.iter())
        .map(|(receiver, channel)| async move {
            choice(
                ctx,
                receiver,
                channel.id,
                0..users.len(),
                |&idx| users[idx].name.clone(),
                '✅'.into(),
            )
            .await
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(ready)
        .fold(vec![0; data.roles.len()], |mut acc, x| {
            acc[x] += 1;
            ready(acc)
        })
        .await;

    {
        let content = format!(
            "Votes:\n{}",
            votes
                .iter()
                .enumerate()
                .map(|(idx, votes)| format!("{}: {}", data.users[idx].name, votes))
                .join("\n")
        );
        let content = &content;
        join_all(data.users.iter().map(|p| p.dm(ctx, |m| m.content(content)))).await;
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
                data.users[voted_players[0]].name.clone()
            )
        } else {
            format!(
                "{} sind gestorben",
                voted_players
                    .iter()
                    .map(|&idx| data.users[idx].name.clone())
                    .join(", ")
            )
        };
        let content = &content;
        join_all(data.users.iter().map(|p| p.dm(ctx, |m| m.content(content)))).await;
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
        join_all(data.users.iter().map(|u| u.dm(ctx, |m| m.content(content)))).await;
    }

    {
        let winners = data
            .users
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

        let content = &format!("Die Gewinner sind: {}", winners);
        join_all(data.users.iter().map(|u| u.dm(ctx, |m| m.content(content)))).await;
    }

    join_all(
        data.users
            .iter()
            .map(|u| u.dm(ctx, |m| m.content("----------------"))),
    )
    .await;

    Ok(())
}

pub struct GameData<'a> {
    pub users: Vec<&'a User>,
    pub dm_channels: Vec<PrivateChannel>,
    pub roles: Vec<Box<dyn Role>>,
    pub extra_roles: Vec<Box<dyn Role>>,
    pub context: &'a Context,
}

async fn setup<'a>(
    mut players: Vec<(&'a User, ReceiverStream<ReactionAction>)>,
    mut player_roles: Vec<Box<dyn Role>>,
    ctx: &'a Context,
) -> (
    GameData<'a>,
    Vec<Box<dyn RoleBehavior>>,
    Vec<ReceiverStream<ReactionAction>>,
) {
    assert!(player_roles.len() >= players.len());

    let (players, extra_roles) = {
        let mut thread_rng = thread_rng();
        let extra_roles: Vec<Box<dyn Role>> = (0..(player_roles.len() - players.len()))
            .map(|_| player_roles.remove((&mut thread_rng).gen_range(0..player_roles.len())))
            .collect();
        players.shuffle(&mut thread_rng);
        (players, extra_roles)
    };
    let mut users = Vec::with_capacity(players.len());
    let mut reactions = Vec::with_capacity(players.len());
    for (user, channel) in players {
        users.push(user);
        reactions.push(channel);
    }
    let dm_channels = users
        .iter()
        .map(|u| u.create_dm_channel(ctx))
        .collect::<FuturesOrdered<_>>()
        .map(|res| res.expect("Error creating DM channel"))
        .collect::<Vec<_>>()
        .await;

    let behaviors = player_roles.iter().map(|r| r.build()).collect::<Vec<_>>();

    (
        GameData {
            context: ctx,
            users,
            roles: player_roles,
            extra_roles,
            dm_channels,
        },
        behaviors,
        reactions,
    )
}

pub async fn choice<T, F, S: ToString>(
    ctx: &Context,
    receiver: &mut ReceiverStream<ReactionAction>,
    channel: ChannelId,
    choices: impl Iterator<Item = T>,
    name: F,
    reaction: ReactionType,
) -> Option<T>
where
    F: Fn(&T) -> S,
{
    let me = ctx.cache.current_user_id().await;

    let (name, reaction) = (&name, &reaction);
    let mut messages = choices
        .map(move |x| async move {
            let msg = channel
                .send_message(ctx, |m| {
                    m.content(name(&x));
                    m.1 = Some(vec![reaction.clone()]);
                    m
                })
                .await;
            match msg {
                Ok(msg) => Some((msg.id, x)),
                Err(e) => {
                    println!("Error sending message: {}", e);
                    None
                }
            }
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(ready)
        .collect::<HashMap<_, _>>()
        .await;

    let mut result = None;

    while let Some(r) = receiver.next().await {
        if messages.contains_key(&r.inner().message_id) && r.inner().user_id.unwrap() != me {
            result = Some(r);
            break;
        }
    }

    let result = match result {
        Some(r) => messages.remove(&r.inner().message_id),
        None => None,
    };

    messages
        .keys()
        .map(|m| channel.delete_message(ctx, m))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    result
}
