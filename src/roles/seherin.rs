use std::iter::once;

use crate::game::select;

use super::*;

pub async fn action<'a>(
    user: &User,
    players: &HashMap<&User, Role>,
    extra_roles: &Vec<Role>,
    ctx: &Context,
) -> CommandResult<Option<Swap<'a>>> {
    user.dm(ctx, |m| m.content("Wesen Rolle willst du sehen?"))
        .await?;

    let others = players.iter().filter(|(&u, _)| u != user);
    let choices = others.map(|(&u, _)| Some(u)).chain(once(None));

    let c = select(
        ctx,
        user.create_dm_channel(ctx).await?.id,
        choices,
        |x| match x {
            Some(u) => u.name.clone(),
            None => "2 Karten aus der Mitte".to_string(),
        },
        '🔮'.into(),
    )
    .await
    .flatten();

    match c {
        Some(u) => {
            user.dm(ctx, |m| {
                m.content(format!(
                    "{} hat die Rolle {}",
                    u.name,
                    players.get(u).expect("player not in map")
                ))
            })
            .await?;
        }
        None => {
            user.dm(ctx, |m| {
                m.content(format!(
                    "2 Rollen in der Mitte sind: {}, {}",
                    extra_roles[0], extra_roles[1]
                ))
            })
            .await?;
        }
    }

    Ok(None)
}
