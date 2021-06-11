use crate::game::choice;

use super::*;

#[derive(Clone, Default)]
pub struct Dieb;

impl Role for Dieb {
    fn build(&self) -> Box<dyn RoleBehavior> {
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
    to_steal: Option<usize>,
}

#[async_trait]
impl RoleBehavior for DiebData {
    async fn ask<'a>(
        &mut self,
        data: &GameData<'a>,
        reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        data.users[index]
            .dm(data.context, |m| m.content("Mit wem willst du tauschen?"))
            .await
            .expect("error sending message");

        let others = data.users.iter().enumerate().filter(|&(i, _)| i != index);

        let to_steal = choice(
            data.context,
            reactions,
            data.dm_channels[index].id,
            others,
            |(_, u)| u.name.clone(),
            '🤚'.into(),
        )
        .await;

        self.to_steal = to_steal.map(|(i, _)| i);
    }

    fn action<'a>(&mut self, data: &mut GameData<'a>, index: usize) {
        if let Some(to_steal) = self.to_steal {
            data.roles.swap(index, to_steal);
        }
    }

    async fn after<'a>(
        &mut self,
        data: &GameData<'a>,
        _reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        if let Some(to_steal) = self.to_steal {
            let name = data.users[to_steal].name.clone();
            let new_role = data.roles[index].to_string();
            data.dm_channels[index]
                .say(data.context, format!("{} war {}", name, new_role))
                .await
                .expect("error sending message");
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
