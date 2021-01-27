use futures::future::{join_all, ready};
use itertools::Itertools;
use rand::prelude::*;
use serenity::{
    framework::standard::CommandResult,
    futures::stream::{FuturesUnordered, StreamExt},
    model::{channel::ReactionType, id::ChannelId, prelude::User},
    prelude::*,
};
use std::collections::HashMap;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    controller::ReactionAction,
    roles::{self, Group, Role, RoleData, Team},
};

pub async fn start_game(
    ctx: &Context,
    players: HashMap<&User, ReceiverStream<ReactionAction>>,
) -> CommandResult {
    let mut player_roles: Vec<Box<dyn Role>> = vec![
        Box::new(roles::Doppel),
        Box::new(roles::Werwolf),
        Box::new(roles::Werwolf),
        Box::new(roles::Dorfbewohner),
        Box::new(roles::Seherin),
        Box::new(roles::Dieb),
        Box::new(roles::Unruhestifterin),
        Box::new(roles::Schlaflose),
    ];
    assert!(player_roles.len() >= players.len());

    let roles_string = player_roles.iter().join(" | ");

    join_all(players.iter().map(|(player, _)| {
        player.dm(ctx, |m| {
            m.content(format!("Die Rollen sind:\n{}", roles_string))
        })
    }))
    .await;

    let (mut players, extra_roles) = {
        let mut thread_rng = thread_rng();
        let extra_roles: Vec<Box<dyn Role>> = (0..(player_roles.len() - players.len()))
            .map(|_| player_roles.remove((&mut thread_rng).gen_range(0..player_roles.len())))
            .collect();
        let mut players = players.into_iter().collect::<Vec<_>>();
        players.shuffle(&mut thread_rng);
        (players, extra_roles)
    };

    let mut roles: HashMap<&User, Box<dyn Role>> = HashMap::with_capacity(players.len());

    let mut players: Vec<(Box<dyn RoleData>, &User, _)> = player_roles
        .iter()
        .map(|role| {
            let (p, m) = players.pop().unwrap();
            roles.insert(p, role.clone());
            (role.build(), p, m)
        })
        .collect();

    join_all(players.iter().map(|(role, u, _)| {
        u.dm(ctx, move |m| {
            m.content(format!("Deine Rolle ist: {}", role))
        })
    }))
    .await;

    // --- Action

    players
        .iter_mut()
        .map(|(role, player, receiver)| role.ask(player, &roles, &extra_roles, ctx, receiver))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    for (role, player, _) in players.iter() {
        role.action(player, &mut roles, &extra_roles, ctx);
    }

    for (role, player, _) in players.iter() {
        role.after(player, &mut roles, &extra_roles, ctx);
    }

    // --- Voting

    join_all(
        players
            .iter()
            .map(|(_, p, _)| p.dm(ctx, |m| m.content("Voting started"))),
    )
    .await;

    let roles = &roles;
    let votes: HashMap<&User, u32> = players
        .iter_mut()
        .map(|(_, player, receiver)| async move {
            choice(
                ctx,
                receiver,
                player
                    .create_dm_channel(ctx)
                    .await
                    .expect("error creating dm channel")
                    .id,
                roles.keys(),
                |p| p.name.clone(),
                '✅'.into(),
            )
            .await
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(ready)
        .fold(HashMap::new(), |mut acc: HashMap<&User, u32>, x| {
            *(acc.entry(x).or_insert(0)) += 1;
            ready(acc)
        })
        .await;

    {
        let content = format!(
            "Votes:\n{}",
            votes
                .iter()
                .map(|(&player, votes)| format!("{}: {}", player.name, votes))
                .join("\n")
        );
        let content = &content;
        join_all(roles.keys().map(|p| p.dm(ctx, |m| m.content(content)))).await;
    }

    let mut skipped = false;
    let voted_players = votes.iter().fold(
        (Vec::new(), 1_u32),
        |(mut result, max), (&player, &votes)| match votes.cmp(&max) {
            std::cmp::Ordering::Less => {
                skipped = true;
                (result, max)
            }
            std::cmp::Ordering::Equal => {
                result.push(player);
                (result, max)
            }
            std::cmp::Ordering::Greater => {
                skipped = true;
                (vec![player], votes)
            }
        },
    );
    let voted_players = if skipped { voted_players.0 } else { vec![] };

    {
        let content = if voted_players.is_empty() {
            "Niemand ist gestorben".to_string()
        } else if voted_players.len() == 1 {
            format!("{} ist gestorben", voted_players[0].name.clone())
        } else {
            format!(
                "{} sind gestorben",
                voted_players.iter().map(|p| p.name.clone()).join(", ")
            )
        };
        let content = &content;
        join_all(roles.keys().map(|p| p.dm(ctx, |m| m.content(content)))).await;
    }

    let has_werewolf = roles.values().any(|role| role.group() == Group::Wolf);

    #[allow(clippy::collapsible_if)]
    let winning_team = if has_werewolf {
        if voted_players
            .iter()
            .any(|p| roles.get(p).expect("player should be in map").group() == Group::Wolf)
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
        join_all(
            players
                .iter()
                .map(|&(_, p, _)| p.dm(ctx, |m| m.content(content))),
        )
        .await;
    }

    {
        let winners = roles
            .iter()
            .filter_map(|(player, role)| {
                if role.team() == winning_team {
                    Some(player.name.clone())
                } else {
                    None
                }
            })
            .join(", ");

        let content = &format!("Die Gewinner sind: {}", winners);
        join_all(
            players
                .iter()
                .map(|&(_, p, _)| p.dm(ctx, |m| m.content(content))),
        )
        .await;
    }

    join_all(
        players
            .iter()
            .map(|&(_, p, _)| p.dm(ctx, |m| m.content("----------------"))),
    )
    .await;

    Ok(())
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
