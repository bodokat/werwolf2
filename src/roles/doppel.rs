use crate::game::choice;

use super::*;

pub struct Doppel;

impl Display for Doppel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

#[async_trait]
impl Role for Doppel {
    async fn action<'a>(
        &self,
        player: &'a User,
        player_roles: &HashMap<&'a User, &Box<dyn Role>>,
        extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        receiver: &mut ReceiverStream<ReactionAction>,
    ) -> CommandResult<Vec<Action<'a>>> {
        player
            .dm(ctx, |m| m.content("Wen willst du kopieren?"))
            .await
            .expect("error sending message");

        let others = player_roles.iter().filter(|(&u, _)| u != player);

        let to_swap: Option<(&&User, &&Box<dyn Role>)> = choice(
            ctx,
            receiver,
            player.create_dm_channel(ctx).await?.id,
            others,
            |(u, _)| u.name.clone(),
            '🤚'.into(),
        )
        .await;

        match to_swap {
            Some((u, role)) => {
                let mut action = role
                    .action(player, player_roles, extra_roles, ctx, receiver)
                    .await?;
                let mut res = vec![
                    Action::Copy {
                        from: u,
                        to: player,
                    },
                    Action::SayRole(player),
                ];
                res.append(&mut action);
                Ok(res)
            }
            None => Ok(vec![]),
        }
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}
