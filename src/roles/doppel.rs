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

    fn name(&self) -> String {
        "Doppelgängerin".into()
    }
}

impl Display for Doppel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

pub struct DoppelData {
    copied: Option<(&'static dyn Role, Box<dyn RoleBehavior>)>,
}

impl Display for DoppelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

#[async_trait]
impl RoleBehavior for DoppelData {
    async fn before_ask<'a>(&mut self, data: &GameData<'a>, index: usize) {
        let others = data
            .players
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != index)
            .collect::<Vec<_>>();

        let to_copy = data.players[index]
            .choice(
                "Wen willst du kopieren?".into(),
                others.iter().map(|(_, u)| u.name.clone()).collect(),
            )
            .await;

        let behavior = data.roles[to_copy].build();
        data.players[index].say(format!("Du bist jetzt {}", data.roles[to_copy]));
        self.copied = Some((data.roles[to_copy].clone(), behavior));
    }

    fn before_action<'a>(&mut self, data: &mut GameData<'a>, index: usize) {
        if let Some((role, _)) = self.copied {
            data.roles[index] = role;
        }
    }

    async fn ask<'a>(&mut self, data: &GameData<'a>, index: usize) {
        if let Some((_, behavior)) = &mut self.copied {
            behavior.ask(data, index).await
        }
    }

    fn action<'a>(&mut self, data: &mut GameData<'a>, index: usize) {
        if let Some((_, behavior)) = &mut self.copied {
            behavior.action(data, index)
        }
    }

    async fn after<'a>(&mut self, data: &GameData<'a>, index: usize) {
        if let Some((_, c)) = &mut self.copied {
            c.after(data, index).await
        }
    }
}
