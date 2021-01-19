use crate::game::choice;
use crate::utils::MapExt;

use super::*;

#[derive(Clone, Default)]
pub struct Dieb;

impl Role for Dieb {
    fn build(&self) -> Box<dyn RoleData> {
        Box::new(DiebData { to_steal: None })
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

#[derive(Clone)]
struct DiebData {
    to_steal: Option<User>,
}

#[async_trait]
impl RoleData for DiebData {
    async fn ask(
        &mut self,
        player: &User,
        player_roles: &HashMap<&User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        receiver: &mut ReceiverStream<ReactionAction>,
    ) {
        player
            .dm(ctx, |m| m.content("Mit wem willst du tauschen?"))
            .await
            .expect("error sending message");

        let others = player_roles.iter().filter(|(&u, _)| u != player);

        let to_steal = choice(
            ctx,
            receiver,
            player.create_dm_channel(ctx).await.unwrap().id,
            others,
            |(u, _)| u.name.clone(),
            '🤚'.into(),
        )
        .await;

        self.to_steal = to_steal.map(|(&u, _)| u.clone());
    }

    fn action(
        &self,
        player: &User,
        player_roles: &mut HashMap<&User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
    ) {
        if let Some(to_steal) = &self.to_steal {
            player_roles.swap(player, &to_steal);
            let ctx = ctx.clone();
            let player_id = player.id;
            let name = to_steal.name.clone();
            let new_role = player_roles.get(player).unwrap().to_string();
            tokio::spawn(async move {
                let _ = player_id
                    .create_dm_channel(&ctx)
                    .await
                    .unwrap()
                    .say(&ctx, format!("{} war {}", name, new_role))
                    .await;
            });
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

impl Display for DiebData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dieb")
    }
}
