use std::iter::once;

use crate::game::choice;

use super::*;

#[derive(Clone, Default)]
pub struct Seherin;

#[async_trait]
impl RoleData for Seherin {
    async fn ask(
        &mut self,
        player: &User,
        players: &HashMap<&User, Box<dyn Role>>,
        extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        receiver: &mut ReceiverStream<ReactionAction>,
    ) {
        player
            .dm(ctx, |m| m.content("Wesen Rolle willst du sehen?"))
            .await
            .unwrap();

        let others = players.iter().filter(|(&u, _)| u != player);
        let choices = others.map(|(&u, _)| Some(u)).chain(once(None));

        let c = choice(
            ctx,
            receiver,
            player.create_dm_channel(ctx).await.unwrap().id,
            choices,
            |x| match x {
                Some(u) => u.name.clone(),
                None => "2 Karten aus der Mitte".to_string(),
            },
            '🔮'.into(),
        )
        .await
        .flatten();

        match c {
            Some(u) => {
                player
                    .dm(ctx, |m| {
                        m.content(format!(
                            "{} hat die Rolle {}",
                            u.name,
                            players.get(u).expect("player not in map")
                        ))
                    })
                    .await
                    .unwrap();
            }
            None => {
                player
                    .dm(ctx, |m| {
                        m.content(format!(
                            "2 Rollen in der Mitte sind: {}, {}",
                            extra_roles[0], extra_roles[1]
                        ))
                    })
                    .await
                    .unwrap();
            }
        }
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for Seherin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Seherin")
    }
}
