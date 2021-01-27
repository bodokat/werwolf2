use super::*;

use crate::utils::MapExt;

#[derive(Clone)]
pub struct Unruhestifterin;

impl Display for Unruhestifterin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unruhestifterin")
    }
}

impl Role for Unruhestifterin {
    fn build(&self) -> Box<dyn RoleData> {
        Box::new(UnruhestifterinData { to_swap: None })
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

#[derive(Clone)]
pub struct UnruhestifterinData {
    to_swap: Option<(User, User)>,
}

#[async_trait]
impl RoleData for UnruhestifterinData {
    async fn ask(
        &mut self,
        player: &User,
        players: &HashMap<&User, Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        receiver: &mut ReceiverStream<ReactionAction>,
    ) {
        player
            .dm(ctx, |m| m.content("Welche Spieler willst du tauschen?"))
            .await
            .unwrap();

        let me = ctx.cache.current_user_id().await;

        let others = players.keys().filter(|&&u| u != player);

        let messages = others
            .map(move |&u| async move {
                let msg = player
                    .dm(ctx, |m| {
                        m.content(u.name.clone());
                        m.1 = Some(vec!['🔁'.into()]);
                        m
                    })
                    .await
                    .expect("error sending message");

                (msg.id, u)
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<HashMap<_, _>>()
            .await;

        let mut to_swap = None::<&User>;

        while let Some(r) = receiver.next().await {
            if messages.contains_key(&r.inner().message_id) && r.inner().user_id.unwrap() != me {
                if let Some(&target) = messages.get(&r.inner().message_id) {
                    match r {
                        ReactionAction::Added(_) => match to_swap {
                            Some(to_swap) if to_swap != target => {
                                self.to_swap = Some((target.clone(), to_swap.clone()));
                                return;
                            }
                            None => to_swap = Some(target),
                            Some(_) => (),
                        },
                        ReactionAction::Removed(_) => match to_swap {
                            Some(t) if t == target => to_swap = None,
                            _ => (),
                        },
                    }
                }
            }
        }
    }

    fn action<'a>(
        &self,
        _player: &'a User,
        player_roles: &mut HashMap<&User, Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        _ctx: &Context,
    ) {
        if let Some((x, y)) = &self.to_swap {
            player_roles.swap(x, y);
        }
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for UnruhestifterinData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unruhestifterin")
    }
}
