use super::*;

use rand::prelude::{thread_rng, IteratorRandom};

pub struct Werwolf;

impl Display for Werwolf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Werwolf")
    }
}

#[async_trait]
impl Role for Werwolf {
    async fn action<'a>(
        &self,
        player: &'a User,
        players: &HashMap<&'a User, &Box<dyn Role>>,
        extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        _receiver: &mut ReceiverStream<ReactionAction>,
    ) -> CommandResult<Vec<Action<'a>>> {
        let mut others = players
            .iter()
            .filter(|(&other_user, role)| role.group() == Group::Wolf && other_user != player);

        let content = match others.next() {
            Some((x, _)) => format!(
                "Die anderen Werwölfe sind: {}, {}",
                x.name.clone(),
                others.map(|(u, _)| u.name.clone()).format(", ")
            ),
            None => match extra_roles
                .iter()
                .filter(|r| r.group() != Group::Wolf)
                .choose(&mut thread_rng())
            {
                Some(x) => format!("Du bist alleine. Eine Karte aus der Mitte ist: {}", x),
                None => "Du bist alleine. Es sind nur Werwölfe in der Mitte".to_string(),
            },
        };

        player.dm(ctx, |m| m.content(content)).await?;
        Ok(vec![])
    }

    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Wolf
    }
}
