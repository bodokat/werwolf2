use futures::{
    future::{join_all, ready},
    stream::FuturesOrdered,
};
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
    roles::{self, Group, Role, Team},
};

#[derive(Clone, Copy)]
enum VoteResult<'a> {
    Tie,
    Kill(&'a User),
}

pub async fn start_game(
    ctx: &Context,
    players: HashMap<&User, ReceiverStream<ReactionAction>>,
) -> CommandResult {
    let ctx = &ctx;

    let mut player_roles: Vec<Box<dyn Role>> = vec![
        Box::new(roles::Werwolf),
        Box::new(roles::Werwolf),
        Box::new(roles::Dorfbewohner),
        Box::new(roles::Seherin),
        Box::new(roles::Dieb),
        Box::new(roles::Unruhestifterin),
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

    let mut roles: HashMap<&User, &Box<dyn Role>> = HashMap::with_capacity(players.len());

    let mut players: Vec<(&Box<dyn Role>, &User, _)> = player_roles
        .iter()
        .map(|role| {
            let (p, m) = players.pop().unwrap();
            roles.insert(p, role);
            (role, p, m)
        })
        .collect();

    join_all(players.iter().map(|(role, u, _)| {
        u.dm(ctx, move |m| {
            m.content(format!("Deine Rolle ist: {}", role))
        })
    }))
    .await;

    // --- Action

    let mut final_roles = roles.clone();

    players
        .iter_mut()
        // perform each role's action
        .map(|(role, player, receiver)| role.action(player, &roles, &extra_roles, ctx, receiver))
        .collect::<FuturesOrdered<_>>()
        .map(|s| {
            s.unwrap_or_else(|err| {
                println!("Error: {}", err);
                vec![]
            })
        })
        .for_each(|actions| {
            actions
                .into_iter()
                .for_each(|action| action.perform(&mut final_roles, ctx));
            ready(())
        })
        .await;

    // --- Voting

    join_all(
        roles
            .keys()
            .map(|p| p.dm(ctx, |m| m.content("Voting started"))),
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

    let vote_result = votes.iter().fold(
        (VoteResult::Tie, 0u32),
        |(result, max), (&player, &votes)| match votes.cmp(&max) {
            std::cmp::Ordering::Less => (result, max),
            std::cmp::Ordering::Equal => (VoteResult::Tie, max),
            std::cmp::Ordering::Greater => (VoteResult::Kill(player), votes),
        },
    );
    let vote_result = vote_result.0;

    {
        let content = match vote_result {
            VoteResult::Tie => "Tie: Nobody was killed".to_string(),
            VoteResult::Kill(p) => format!("{} was killed", p.name),
        };
        let content = &content;
        join_all(roles.keys().map(|p| p.dm(ctx, |m| m.content(content)))).await;
    }

    let has_werewolf = final_roles.values().any(|role| role.group() == Group::Wolf);

    let winning_team = if has_werewolf {
        match vote_result {
            VoteResult::Kill(p)
                if final_roles.get(p).expect("player should be in map").group() == Group::Wolf =>
            {
                Team::Dorf
            }
            _ => Team::Wolf,
        }
    } else {
        match vote_result {
            VoteResult::Tie => Team::Dorf,
            VoteResult::Kill(_) => Team::Wolf,
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

    let winners = final_roles
        .iter()
        .filter(|(_, role)| role.team() == winning_team)
        .map(|(player, _)| player.name.clone())
        .join(", ");

    {
        let content = &format!("Die Gewinner sind: {}\n----------", winners);
        join_all(
            players
                .iter()
                .map(|&(_, p, _)| p.dm(ctx, |m| m.content(content))),
        )
        .await;
    }

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
