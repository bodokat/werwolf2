use futures::future::{join_all, ready};
use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use serenity::{
    collector::ReactionCollectorBuilder,
    framework::standard::CommandResult,
    futures::stream::{FuturesUnordered, StreamExt},
    futures::FutureExt,
    model::{channel::ReactionType, id::ChannelId, prelude::User},
    prelude::*,
};
use std::collections::HashMap;

use crate::{lobby::Lobby, roles::Role};

#[derive(Clone, Copy)]
enum VoteResult<'a> {
    Tie,
    Kill(&'a User),
}

pub struct Swap<'a>(pub &'a User, pub &'a User);

pub async fn start_game(ctx: &Context, Lobby { players, .. }: &Lobby) -> CommandResult {
    let ctx = &ctx;

    let mut roles = vec![
        Role::Werwolf,
        Role::Werwolf,
        Role::Dorfbewohner,
        Role::Dieb,
        Role::Seherin,
        Role::Unruhestifterin,
    ];
    assert!(roles.len() >= players.len());

    let roles_string = roles.iter().join(" | ");

    join_all(players.iter().map(|p| {
        p.dm(ctx, |m| {
            m.content(format!("Die Rollen sind:\n{}", roles_string))
        })
    }))
    .await;

    // guarantee that there is at least 1 werewolf
    // roles[1..].shuffle(&mut thread_rng());
    roles.shuffle(&mut thread_rng());

    let mut roles_iter = roles.into_iter();

    let mut players: HashMap<&User, Role> = players
        .iter()
        .map(|user| (user, roles_iter.next().expect("not enough roles")))
        .collect::<HashMap<_, _>>();

    let extra_roles = roles_iter.collect::<Vec<_>>();

    join_all(players.iter().map(|(&u, &role)| {
        u.dm(ctx, move |m| {
            m.content(format!("Deine Rolle ist: {}", role))
        })
    }))
    .await;

    // --- Action

    join_all(
        players
            .keys()
            .map(|p| p.dm(ctx, |m| m.content("Game has started"))),
    )
    .await;

    let mut swaps: Vec<(Role, Swap)> = {
        players
            .iter()
            .map(|(&player, role)| {
                role.action(player, &players, &extra_roles, ctx)
                    .map(move |swap| (role, swap))
            })
            .collect::<FuturesUnordered<_>>()
            .map(|(role, s)| match s {
                Ok(s) => (role, s),
                Err(e) => {
                    println!("Error: {}", e);
                    (role, None)
                }
            })
            .filter_map(|(&role, swap)| ready(swap.map(|s| (role, s))))
            .collect::<Vec<_>>()
            .await
    };
    swaps.sort_unstable_by_key(|&(x, _)| x);

    let final_roles = players.clone();

    for (_, Swap(u1, u2)) in swaps.iter() {
        let a = players.get_mut(u1).unwrap() as *mut Role;
        let b = players.get_mut(u2).unwrap() as *mut Role;
        unsafe {
            std::ptr::swap(a, b);
        }
    }

    // --- Voting

    join_all(
        players
            .keys()
            .map(|p| p.dm(ctx, |m| m.content("Voting started"))),
    )
    .await;

    let players = &players;

    let votes: HashMap<&User, u32> = players
        .keys()
        .map(|&player| async move {
            select(
                ctx,
                player
                    .create_dm_channel(ctx)
                    .await
                    .expect("error creating dm channel")
                    .id,
                players.keys(),
                |p| p.name.clone(),
                '✅'.into(),
            )
            .await
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(|x| ready(x))
        .fold(HashMap::new(), |mut acc: HashMap<&User, u32>, x| {
            *(acc.entry(x).or_insert(0)) += 1;
            async move { acc }
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
        join_all(players.keys().map(|p| p.dm(ctx, |m| m.content(content)))).await;
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
        join_all(players.keys().map(|p| p.dm(ctx, |m| m.content(content)))).await;
    }

    let has_werewolf = final_roles
        .values()
        .find(|&&x| x == Role::Werwolf)
        .is_some();

    {
        let content = if has_werewolf {
            match vote_result {
                VoteResult::Kill(p)
                    if *final_roles.get(p).expect("player should be in map") == Role::Werwolf =>
                {
                    "Dorf hat gewonnen"
                }
                _ => "Werwölfe haben gewonnen",
            }
        } else {
            match vote_result {
                VoteResult::Tie => "Dorf het gewonnen",
                VoteResult::Kill(_) => "Werwölfe haben gewonnen",
            }
        };
        join_all(players.keys().map(|p| p.dm(ctx, |m| m.content(content)))).await;
    }

    Ok(())
}

pub async fn select<T, F, S: ToString>(
    ctx: &Context,
    channel: ChannelId,
    choices: impl Iterator<Item = T>,
    name: F,
    reaction: ReactionType,
) -> Option<T>
where
    F: Fn(&T) -> S,
{
    let collector = ReactionCollectorBuilder::new(ctx).channel_id(channel).await;
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
        .filter_map(|x| ready(x))
        .collect::<HashMap<_, _>>()
        .await;

    let me = ctx.cache.current_user_id().await;
    let mut collector = collector.filter(|r| {
        ready(
            messages.contains_key(&r.as_inner_ref().message_id)
                && r.as_inner_ref().user_id.unwrap() != me,
        )
    });

    let result = match collector.next().await {
        Some(r) => messages.remove(&r.as_inner_ref().message_id),
        None => {
            println!("Got no reaction");
            None
        }
    };

    messages
        .keys()
        .map(|m| channel.delete_message(ctx, m))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    result
}
