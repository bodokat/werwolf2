use super::*;

pub struct Unruhestifterin;

#[async_trait]
impl Role for Unruhestifterin {
    async fn action<'a>(
        &self,
        player: &'a User,
        players: &HashMap<&'a User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        receiver: &mut ReceiverStream<ReactionAction>,
    ) -> CommandResult<Vec<Action<'a>>> {
        player
            .dm(ctx, |m| m.content("Welche Spieler willst du tauschen?"))
            .await?;

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
                                return Ok(vec![Action::Swap(target, to_swap)])
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

        Ok(vec![])
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for Unruhestifterin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unruhestifterin")
    }
}
