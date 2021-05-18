use super::*;

#[derive(Clone)]
pub struct Unruhestifterin;

impl Display for Unruhestifterin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unruhestifterin")
    }
}

impl Role for Unruhestifterin {
    fn build(&self) -> Box<dyn RoleBehavior> {
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
    to_swap: Option<(usize, usize)>,
}

#[async_trait]
impl RoleBehavior for UnruhestifterinData {
    async fn ask<'a>(
        &mut self,
        data: &GameData<'a>,
        reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        data.dm_channels[index]
            .say(data.context, "Welche Spieler willst du tauschen?")
            .await
            .expect("Error sending message");

        let others = data.users.iter().enumerate().filter(|&(i, _)| i != index);
        let me = data.context.cache.current_user_id().await;

        let messages = others
            .map(move |(u, _)| async move {
                let msg = data.dm_channels[index]
                    .send_message(data.context, |m| {
                        m.content(data.users[u].name.clone());
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

        let mut to_swap = None::<usize>;

        while let Some(r) = reactions.next().await {
            if r.inner().user_id.unwrap() != me {
                if let Some(&target) = messages.get(&r.inner().message_id) {
                    match r {
                        ReactionAction::Added(_) => match to_swap {
                            Some(to_swap) if to_swap != target => {
                                self.to_swap = Some((target, to_swap));
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

    fn action<'a>(&mut self, data: &mut GameData<'a>, _index: usize) {
        if let Some((x, y)) = self.to_swap {
            data.roles.swap(x, y);
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
