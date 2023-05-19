use itertools::Itertools;

use super::{async_trait, Data, Display, Group, Role, RoleBehavior, Team};

use std::iter;

pub static Freimaurer: &'static dyn Role = &FreimaurerImpl;

#[derive(Clone, Default)]
struct FreimaurerImpl;

impl Display for FreimaurerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Freimaurer")
    }
}

impl Role for FreimaurerImpl {
    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(FreimaurerImpl)
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn name(&self) -> String {
        "Freimaurer".into()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl RoleBehavior for FreimaurerImpl {
    async fn ask<'a>(&mut self, data: &Data<'a>, index: usize) {
        let mut others = data
            .roles
            .iter()
            .enumerate()
            .filter(|&(_, &r)| r.as_any().is::<FreimaurerImpl>());

        let content = match others.next() {
            Some((x, _)) => format!(
                "Die anderen Freimaurer sind: {}",
                iter::once(data.players[x].name.clone())
                    .chain(others.map(|(u, _)| data.players[u].name.clone()))
                    .format(", ")
            ),
            None => "Du bist alleine.".to_string(),
        };

        data.players[index].say(content);
    }
}
