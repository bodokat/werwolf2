use super::{async_trait, Data, Display, Group, Role, RoleBehavior, Team};

pub static Doppel: &'static dyn Role = &DoppelImpl;

struct DoppelImpl;

impl Role for DoppelImpl {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DoppelImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

struct DoppelData {
    copied: Option<(&'static dyn Role, Box<dyn RoleBehavior>)>,
}

impl Display for DoppelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Doppelgängerin")
    }
}

#[async_trait]
impl RoleBehavior for DoppelData {
    async fn before_ask<'a>(&mut self, data: &Data<'a>, index: usize) {
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
        data.players[index].say(format!("Du bist jetzt {}", data.roles[to_copy].name()));
        self.copied = Some((data.roles[to_copy], behavior));
    }

    fn before_action(&mut self, data: &mut Data<'_>, index: usize) {
        if let Some((role, _)) = self.copied {
            data.roles[index] = role;
        }
    }

    async fn ask<'a>(&mut self, data: &Data<'a>, index: usize) {
        if let Some((_, behavior)) = &mut self.copied {
            behavior.ask(data, index).await;
        }
    }

    fn action(&mut self, data: &mut Data<'_>, index: usize) {
        if let Some((_, behavior)) = &mut self.copied {
            behavior.action(data, index);
        }
    }

    async fn after<'a>(&mut self, data: &Data<'a>, index: usize) {
        if let Some((_, c)) = &mut self.copied {
            c.after(data, index).await;
        }
    }
}
