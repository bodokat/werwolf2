use std::iter;

use itertools::Itertools;

use super::{async_trait, Data, Display, Group, Role, RoleBehavior, Team};

pub static Gunstling: &'static dyn Role = &GunstlingImpl;

#[derive(Clone, Default)]
struct GunstlingImpl;

impl Role for GunstlingImpl {
    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(GunstlingImpl)
    }

    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn name(&self) -> String {
        "Günstling".into()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl RoleBehavior for GunstlingImpl {
    async fn ask<'a>(&mut self, data: &Data<'a>, index: usize) {
        let mut wolves = data
            .roles
            .iter()
            .enumerate()
            .filter(|(_, role)| role.group() == Group::Wolf);

        let content = match wolves.next() {
            Some((x, _)) => format!(
                "Die anderen Freimaurer sind: {}",
                iter::once(data.players[x].name.clone())
                    .chain(wolves.map(|(u, _)| data.players[u].name.clone()))
                    .format(", ")
            ),
            None => "Du bist alleine.".to_string(),
        };

        data.players[index].say(content);
    }
}

impl Display for GunstlingImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Günstling")
    }
}
