use futures::{
    future::{join_all, ready},
    pin_mut,
    stream::FuturesOrdered,
};
use itertools::Itertools;
use rand::prelude::*;
use serenity::{
    futures::stream::{FuturesUnordered, StreamExt},
    model::{
        channel::PrivateChannel,
        id::ChannelId,
        interactions::{Interaction, InteractionData},
        prelude::User,
    },
    prelude::*,
};

use tokio_stream::wrappers::ReceiverStream;

use crate::roles::{self, Group, Role, RoleBehavior, Team};

pub async fn start_game(ctx: &Context, players: Vec<(&User, ReceiverStream<Interaction>)>) {
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

    let (mut data, mut behaviors, mut reactions) = setup(players, roles, ctx).await;

    let roles_string = data.roles.iter().join(" | ");

    join_all(
        data.dm_channels
            .iter()
            .map(|c| c.say(data.context, format!("Die Rollen sind:\n{}", roles_string))),
    )
    .await;

    join_all(
        data.dm_channels
            .iter()
            .zip(data.roles.iter())
            .map(|(c, role)| c.say(data.context, format!("Deine Rolle ist: {}", role))),
    )
    .await;

    // --- Action

    behaviors
        .iter_mut()
        .zip(reactions.iter_mut())
        .enumerate()
        .map(|(idx, (behavior, reactions))| behavior.before_ask(&data, reactions, idx))
        .collect::<FuturesUnordered<_>>()
        .for_each(|_| ready(()))
        .await;

    behaviors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, behavior)| behavior.before_action(&mut data, idx));

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
        data.dm_channels
            .iter()
            .map(|c| c.say(data.context, "Voting started")),
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
                "Wähle einen Mitspieler",
                0..users.len(),
                |&idx| users[idx].name.clone(),
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
        join_all(data.dm_channels.iter().map(|p| p.say(ctx, content))).await;
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
        join_all(data.dm_channels.iter().map(|p| p.say(ctx, content))).await;
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
        join_all(data.dm_channels.iter().map(|p| p.say(ctx, content))).await;
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
        join_all(data.dm_channels.iter().map(|p| p.say(ctx, content))).await;
    }

    join_all(data.dm_channels.iter().map(|p| p.say(ctx, "-------------"))).await;
}

pub struct GameData<'a> {
    pub users: Vec<&'a User>,
    pub dm_channels: Vec<PrivateChannel>,
    pub roles: Vec<Box<dyn Role>>,
    pub extra_roles: Vec<Box<dyn Role>>,
    pub context: &'a Context,
}

async fn setup<'a>(
    mut players: Vec<(&'a User, ReceiverStream<Interaction>)>,
    mut player_roles: Vec<Box<dyn Role>>,
    ctx: &'a Context,
) -> (
    GameData<'a>,
    Vec<Box<dyn RoleBehavior>>,
    Vec<ReceiverStream<Interaction>>,
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

pub async fn choice<T, F, D: ToString, S: ToString>(
    ctx: &Context,
    receiver: &mut ReceiverStream<Interaction>,
    channel: ChannelId,
    title: D,
    choices: impl Iterator<Item = T>,
    name: F,
) -> Option<T>
where
    F: Fn(&T) -> S,
{
    let mut choices: Vec<T> = choices.collect();

    let stream = receiver.filter_map(|i| async {
        if let Some(InteractionData::MessageComponent(data)) = i.data {
            Some(
                data.custom_id
                    .parse::<usize>()
                    .expect("Error parsing custom_id"),
            )
        } else {
            None
        }
    });

    channel
        .send_message(&ctx, |m| {
            m.content(title);
            m.components(|c| {
                let chunks = choices.iter().enumerate().chunks(5);
                chunks.into_iter().for_each(|chunk| {
                    c.create_action_row(|a| {
                        chunk.for_each(|(index, choice)| {
                            a.create_button(|b| {
                                b.label(name(choice));
                                b.custom_id(index.to_string());
                                b.style(serenity::model::interactions::ButtonStyle::Primary)
                            });
                        });
                        a
                    });
                });
                c
            })
        })
        .await
        .expect("Error sending message");

    let stream = stream.map(move |n| choices.remove(n));
    pin_mut!(stream);

    stream.next().await
}
