use crate::game::choice;

use super::*;

#[derive(Clone)]
pub struct Doppel;

impl Role for Doppel {
    fn build(&self) -> Box<dyn RoleBehavior> {
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
    copied: Option<(Box<dyn Role>, Box<dyn RoleBehavior>)>,
}

impl Display for DoppelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

#[async_trait]
impl RoleBehavior for DoppelData {
    async fn ask<'a>(
        &mut self,
        data: &GameData<'a>,
        reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        data.users[index]
            .dm(data.context, |m| m.content("Wen willst du kopieren?"))
            .await
            .expect("error sending message");

        let others = data.users.iter().enumerate().filter(|&(i, _)| i != index);

        let to_copy = choice(
            data.context,
            reactions,
            data.dm_channels[index].id,
            others,
            |(_, u)| u.name.clone(),
            '🤚'.into(),
        )
        .await;

        if let Some((to_copy, _)) = to_copy {
            let mut behavior = data.roles[to_copy].build();
            let _ = data.dm_channels[index]
                .say(
                    data.context,
                    format!("Du bist jetzt {}", data.roles[to_copy]),
                )
                .await;
            behavior.ask(data, reactions, index).await;
            self.copied = Some((data.roles[to_copy].clone(), behavior));
        }
    }

    fn action<'a>(&mut self, data: &mut GameData<'a>, index: usize) {
        if let Some((role, behavior)) = &mut self.copied {
            data.roles[index] = role.clone();
            behavior.action(data, index)
        }
    }

    async fn after<'a>(
        &mut self,
        data: &GameData<'a>,
        reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        if let Some((_, c)) = &mut self.copied {
            c.after(data, reactions, index).await
        }
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}
