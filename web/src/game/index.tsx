import { Navigate, useLoaderData, useLocation } from "react-router-dom";
import { useNavigate } from "react-router-dom";
import { GameSession, SessionState } from "../api/gameSession";
import { notifications } from "@mantine/notifications";
import { createContext, useState } from "react";
import { useSubject } from "../utils/useSubject";
import { ActionIcon, Box, Button, Group, Stack, Modal, Paper, TextInput, rem, Text } from "@mantine/core";
import { Clipboard } from "tabler-icons-react";
import { Messages } from "./messages";
import { Players } from "./players";
import { Settings } from "./settings";

export let stateContext = createContext<SessionState | null>(null)
export let sessionContext = createContext<GameSession | null>(null)

export function Game() {
    let session = useLoaderData()
    if (!(session instanceof GameSession)) {
        console.log("bruh")
        notifications.show({
            title: "Lobby not found",
            message: "The Lobby you're connecting to was not found"
        })
        return <Navigate to={"/"} />
    }

    let state = useSubject(session.state)
    console.log(state)

    return <sessionContext.Provider value={session}>
        <stateContext.Provider value={state}>
            <Stack justify="flex-start" sx={{ minWidth: "100vw", minHeight: "100vh" }}>


                <Group>
                    Verbunden mit <Paper>{window.location.href} </Paper>
                    <ActionIcon onClick={() => window.navigator.clipboard.writeText(window.location.href)}>
                        <Clipboard />
                    </ActionIcon>
                </Group>


                <Box sx={{ display: "grid", gridTemplateColumns: "3fr 2fr", columnGap: "10px", padding: "2rem" }}>
                    <Box sx={{ gridColumn: 1, gridRowStart: 1, gridRowEnd: 3, border: "2px solid #595959", borderRadius: "5px" }}>
                        {(
                            state.me ? <Messages /> :
                                <NameInput session={session} />
                        )}

                    </Box>
                    <Box sx={{ gridColumn: 2, gridRow: 1, minHeight: "10%", border: "2px solid #595959", borderRadius: "5px" }}>
                        <Players />
                    </Box>
                    <Box sx={{ gridColumn: 2, gridRow: 2, minWidth: rem(25), border: "2px solid #595959", borderRadius: "5px" }}>
                        <Settings />
                    </Box>
                </Box>

            </Stack>
        </stateContext.Provider>
    </sessionContext.Provider>
}

function NameInput({ session }: { session: GameSession }) {
    const [name, setName] = useState("")
    const [loading, setLoading] = useState(false)
    return <>
        <TextInput
            label="Choose name"
            onChange={(event) => setName(event.currentTarget.value)}
            autoFocus />
        <Button variant="filled" disabled={loading} onClick={() => {
            if (!loading) {
                setLoading(true)
                let listener = ({ data }: MessageEvent) => {
                    console.log(data)
                    if (data === "namerejected") {
                        setLoading(false)
                        session.socket.removeEventListener("message", listener)
                    }
                }
                session.socket.addEventListener("message", listener)
                session.socket.send(name)
            }
        }}>
            Set Name
        </Button>
    </>



}