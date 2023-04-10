import { useContext } from "react";
import { stateContext } from ".";
import { Badge, Paper, SimpleGrid, Text } from "@mantine/core";
import React from "react";

export function Players() {
    let state = useContext(stateContext)!
    let players = state.players

    return <SimpleGrid>
        {(players.length == 0 ?
            <Text>No players</Text> :
            players.map((p) =>
                <Paper key={p}>
                    {p}
                    {(state.me === p ? <Badge color="blue">me</Badge> : null)}
                    {(state.settings.admin == p ? <Badge color="red">Admin</Badge> : null)}
                </Paper>
            ))}
    </SimpleGrid>
}