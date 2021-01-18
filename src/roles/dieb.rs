use crate::game::choice;

use super::*;

pub struct Dieb;

#[async_trait]
impl Role for Dieb {
    async fn action<'a>(
        &self,
        player: &'a User,
        player_roles: &HashMap<&'a User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        receiver: &mut ReceiverStream<ReactionAction>,
    ) -> CommandResult<Vec<Action<'a>>> {
        player
            .dm(ctx, |m| m.content("Mit wem willst du tauschen?"))
            .await
            .expect("error sending message");

        let others = player_roles.iter().filter(|(&u, _)| u != player);

        let to_swap = choice(
            ctx,
            receiver,
            player.create_dm_channel(ctx).await?.id,
            others,
            |(u, _)| u.name.clone(),
            '🤚'.into(),
        )
        .await;

        match to_swap {
            Some((u, _)) => Ok(vec![Action::Swap(player, u), Action::SayRole(player)]),
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

impl Display for Dieb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dieb")
    }
}
