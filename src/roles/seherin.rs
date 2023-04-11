use std::iter::once;

use super::{Data, Display, Group, Role, RoleBehavior, Team, async_trait};

#[derive(Clone, Default)]
pub struct Seherin;

impl Role for Seherin {
    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(Seherin)
    }

    fn name(&self) -> String {
        "Seherin".into()
    }
}

#[async_trait]
impl RoleBehavior for Seherin {
    async fn ask<'a>(&mut self, data: &Data<'a>, index: usize) {
        let others = data.players.iter().enumerate().filter(|&(i, _)| i != index);
        let choices = others
            .map(|(u, _)| Some(u))
            .chain(once(None))
            .collect::<Vec<_>>();

        let response = data.players[index].choice(
            "Wesen Rolle willst du sehen?".into(),
            choices
                .iter()
                .map(|&x| match x {
                    Some(u) => data.players[u].name.clone(),
                    None => "2 Karten aus der Mitte".to_string(),
                })
                .collect(),
        );

        match choices[response.await] {
            Some(u) => {
                data.players[index].say(format!(
                    "{} hat die Rolle {}",
                    data.players[u].name, data.roles[u]
                ));
            }
            None => {
                data.players[index].say(format!(
                    "2 Rollen in der Mitte sind: {}, {}",
                    data.extra_roles[0], data.extra_roles[1]
                ));
            }
        }
    }
}

impl Display for Seherin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Seherin")
    }
}
