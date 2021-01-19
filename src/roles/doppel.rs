use crate::game::choice;

use super::*;

#[derive(Clone)]
pub struct Doppel;

impl Role for Doppel {
    fn build(&self) -> Box<dyn RoleData> {
        Box::new(DoppelData { copied: None })
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Wolf
    }
}

impl Display for Doppel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

pub struct DoppelData {
    copied: Option<Box<dyn RoleData>>,
}

impl Display for DoppelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

#[async_trait]
impl RoleData for DoppelData {
    async fn ask(
        &mut self,
        player: &User,
        player_roles: &HashMap<&User, &Box<dyn Role>>,
        extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        receiver: &mut ReceiverStream<ReactionAction>,
    ) {
        player
            .dm(ctx, |m| m.content("Wen willst du kopieren?"))
            .await
            .expect("error sending message");

        let others = player_roles.iter().filter(|(&u, _)| u != player);

        let to_copy: Option<(&&User, &&Box<dyn Role>)> = choice(
            ctx,
            receiver,
            player.create_dm_channel(ctx).await.unwrap().id,
            others,
            |(u, _)| u.name.clone(),
            '🤚'.into(),
        )
        .await;

        if let Some((_, &role)) = to_copy {
            let mut role = role.build();
            role.ask(player, player_roles, extra_roles, ctx, receiver)
                .await;
            self.copied = Some(role);
        }
    }

    fn action(
        &self,
        player: &User,
        player_roles: &mut HashMap<&User, &Box<dyn Role>>,
        extra_roles: &[Box<dyn Role>],
        ctx: &Context,
    ) {
        if let Some(c) = &self.copied {
            c.action(player, player_roles, extra_roles, ctx)
        }
    }

    fn after(
        &self,
        player: &User,
        player_roles: &mut HashMap<&User, &Box<dyn Role>>,
        extra_roles: &[Box<dyn Role>],
        ctx: &Context,
    ) {
        if let Some(c) = &self.copied {
            c.after(player, player_roles, extra_roles, ctx)
        }
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}
