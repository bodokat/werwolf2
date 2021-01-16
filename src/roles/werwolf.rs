use super::*;

use rand::prelude::{thread_rng, IteratorRandom};

pub async fn action<'a>(
    player: &User,
    players: &HashMap<&User, Role>,
    extra_roles: &Vec<Role>,
    ctx: &Context,
) -> CommandResult<Option<Swap<'a>>> {
    let mut others = players
        .iter()
        .filter(|(&other_user, &role)| role == Role::Werwolf && other_user != player);

    let content = match others.next() {
        Some((x, _)) => format!(
            "Die anderen Werwölfe sind: {}, {}",
            x.name.clone(),
            others.map(|(u, _)| u.name.clone()).format(", ")
        ),
        None => match extra_roles
            .iter()
            .filter(|&&r| r != Role::Werwolf)
            .choose(&mut thread_rng())
        {
            Some(x) => format!("Du bist alleine. Eine Karte aus der Mitte ist: {}", x),
            None => format!("Du bist alleine. Es sind nur Werwölfe in der Mitte"),
        },
    };

    player.dm(ctx, |m| m.content(content)).await?;
    Ok(None)
}
