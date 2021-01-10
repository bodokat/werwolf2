use crate::game::select;

use super::*;

pub async fn action<'a>(
    user: &'a User,
    players: &HashMap<&'a User, Role>,
    ctx: &Context,
) -> CommandResult<Option<Swap<'a>>> {
    user.dm(ctx, |m| m.content("Mit wem willst du tauschen?"))
        .await
        .expect("error sending message");

    let others = players.iter().filter(|(&u, _)| u != user);

    let to_swap: Option<(&&User, &Role)> = select(
        ctx,
        user.create_dm_channel(ctx).await?.id,
        others,
        |(u, _)| u.name.clone(),
        '🤚'.into(),
    )
    .await;

    if let Some((u, r)) = to_swap {
        // println!("sende dieb message");
        user.dm(ctx, |m| m.content(format!("{} war {}", u.name, r)))
            .await?;
    }

    Ok(to_swap.map(|(u, _)| Swap(user, u)))
}
