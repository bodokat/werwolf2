import { useContext } from "react";
import { sessionContext, stateContext } from ".";
import React from "react";
import { ActionIcon, Button, Group, NumberInput, Stack, rem } from "@mantine/core";
import { Plus, Minus } from "tabler-icons-react";
import { ToServer } from "../../../bindings/ToServer";

export function Settings() {
    console.log("settings rendered")
    let state = useContext(stateContext)!
    let session = useContext(sessionContext)!
    let isAdmin = state.me != null && state.me === state.settings.admin

    return <>
        <Button disabled={!isAdmin || state.started}
            onClick={() => {
                session.send({ type: "start" })
            }}
        >
            Start
        </Button>
        <Stack>
            {state.settings.available_roles.map((role, index) =>
                <Group position="apart" key={role}>
                    {role}
                    {(isAdmin ?
                        <Group>
                            <ActionIcon variant="outline"
                                disabled={state.settings.roles[index] === 0}
                                onClick={() => {
                                    if (state.settings.roles[index] > 0) {
                                        state.settings.roles[index] -= 1
                                        session.send({
                                            type: "changeroles",
                                            new_roles: state.settings.roles
                                        })
                                    }
                                }}
                            >
                                <Minus />
                            </ActionIcon>

                            <NumberInput hideControls value={state.settings.roles[index]} min={0} onChange={(val) => {
                                if (val != "") {
                                    state.settings.roles[index] = val
                                    session.send({
                                        type: "changeroles",
                                        new_roles: state.settings.roles
                                    })
                                }
                            }}
                                styles={{ input: { width: rem(54), textAlign: 'center' } }}
                            />

                            <ActionIcon variant="outline"
                                onClick={() => {
                                    state.settings.roles[index] += 1
                                    session.send({
                                        type: "changeroles",
                                        new_roles: state.settings.roles
                                    })
                                }}
                            >
                                <Plus />
                            </ActionIcon>

                        </Group> :
                        state.settings.roles[index]
                    )
                    }

                </Group>
            )}
        </Stack>
    </>
}