use std::iter::once;

use crate::game::choice;

use super::*;

#[derive(Clone, Default)]
pub struct Seherin;

#[async_trait]
impl RoleBehavior for Seherin {
    async fn ask<'a>(
        &mut self,
        data: &GameData<'a>,
        reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        data.dm_channels[index]
            .say(data.context, "Wesen Rolle willst du sehen?")
            .await
            .expect("Error sending message");

        let others = data.users.iter().enumerate().filter(|&(i, _)| i != index);
        let choices = others.map(|(u, _)| Some(u)).chain(once(None));

        let c: Option<usize> = choice(
            data.context,
            reactions,
            data.dm_channels[index].id,
            choices,
            |&x| match x {
                Some(u) => data.users[u].name.clone(),
                None => "2 Karten aus der Mitte".to_string(),
            },
            '🔮'.into(),
        )
        .await
        .flatten();

        match c {
            Some(u) => {
                data.dm_channels[index]
                    .say(
                        data.context,
                        format!("{} hat die Rolle {}", data.users[u].name, data.roles[u]),
                    )
                    .await
                    .expect("Error sending message");
            }
            None => {
                data.dm_channels[index]
                    .say(
                        data.context,
                        format!(
                            "2 Rollen in der Mitte sind: {}, {}",
                            data.extra_roles[0], data.extra_roles[1]
                        ),
                    )
                    .await
                    .expect("Error sending message");
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
