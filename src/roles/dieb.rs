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

    fn name(&self) -> String {
        "Dieb".into()
    }
}

#[derive(Clone)]
struct DiebData {
    to_steal: Option<usize>,
}

#[async_trait]
impl RoleBehavior for DiebData {
    async fn ask<'a>(&mut self, data: &GameData<'a>, index: usize) {
        let others = data
            .players
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != index)
            .collect::<Vec<_>>();

        let to_steal = data.players[index]
            .choice(
                "Mit wem willst du tauschen?".into(),
                others.iter().map(|(_, u)| u.name.clone()).collect(),
            )
            .await;

        self.to_steal = Some(others[to_steal].0);
    }

    fn action<'a>(&mut self, data: &mut GameData<'a>, index: usize) {
        if let Some(to_steal) = self.to_steal {
            data.roles.swap(index, to_steal);
        }
    }

    async fn after<'a>(&mut self, data: &GameData<'a>, index: usize) {
        if let Some(to_steal) = self.to_steal {
            let name = data.players[to_steal].name.clone();
            let new_role = data.roles[index].to_string();
            data.players[index].say(format!("{} war {}", name, new_role))
        }
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
