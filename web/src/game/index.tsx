import { Navigate, useLoaderData, useLocation } from "react-router-dom";
import { useNavigate } from "react-router-dom";
import { GameSession, GameState } from "../api/gameSession";
import { notifications } from "@mantine/notifications";
import { createContext, useState } from "react";
import { useSubject } from "../utils/useSubject";
import { ActionIcon, Box, Button, Group, Stack, Modal, Paper, TextInput, rem, Text } from "@mantine/core";
import { Clipboard } from "tabler-icons-react";
import { Messages } from "./messages";
import { Players } from "./players";
import { Settings } from "./settings";

export let stateContext = createContext<GameState | null>(null)
export let sessionContext = createContext<GameSession | null>(null)

export function Game() {
    let session = useLoaderData()
    if (!(session instanceof GameSession)) {
        notifications.show({
            title: "Lobby not found",
            message: "The Lobby you're connecting to was not found"
        })
        return <Navigate to={"/"} />
    }

    let state = useSubject(session.state)

    return <>
        <NameInput session={session} state={state} />
        <sessionContext.Provider value={session}>
            <stateContext.Provider value={state}>
                <Box sx={{ width: "100vw", height: "100vh", display: "grid", gridTemplateRows: "auto 1fr" }}>

                    <Group sx={{ padding: "1rem" }}>
                        Verbunden mit {window.location.href}
                        <ActionIcon onClick={() => window.navigator.clipboard.writeText(window.location.href)}>
                            <Clipboard />
                        </ActionIcon>
                    </Group>


                    <Box sx={{ display: "grid", gridTemplateColumns: "3fr 2fr", columnGap: "10px", padding: "2rem", overflow: "auto" }}>
                        <Box sx={{ gridColumn: 1, gridRowStart: 1, gridRowEnd: 3, border: "2px solid #595959", borderRadius: "5px", padding: "1rem", overflow: "auto" }}>
                            <Messages />
                        </Box>
                        <Box sx={{ gridColumn: 2, gridRow: 1, minHeight: "10%", border: "2px solid #595959", borderRadius: "5px", padding: "1rem", overflow: "auto" }}>
                            <Players />
                        </Box>
                        <Box sx={{ gridColumn: 2, gridRow: 2, minWidth: rem(25), border: "2px solid #595959", borderRadius: "5px", padding: "1rem", overflow: "auto" }}>
                            <Settings />
                        </Box>
                    </Box>

                </Box>
            </stateContext.Provider>
        </sessionContext.Provider>
    </>
}

function NameInput({ session, state }: { session: GameSession, state: GameState }) {
    const [name, setName] = useState("")
    const [loading, setLoading] = useState(false)
    return <Modal opened={state.me == null} onClose={() => { }} closeOnEscape={false} closeOnClickOutside={false} withCloseButton={false}>
        <TextInput
            label="Choose name"
            disabled={loading}
            value={name}
            onChange={(event) => setName(event.currentTarget.value)}
            autoFocus />
        <Button variant="filled" disabled={loading} onClick={() => {
            if (!loading && name != "") {
                setLoading(true)
                let listener = ({ data }: MessageEvent) => {
                    if (data === "namerejected") {
                        setLoading(false)
                        session.socket.removeEventListener("message", listener)
                    }
                }
                let subscription = session.messages.subscribe(m => {
                    if (m.type === "namerejected") {
                        console.log("test")
                        notifications.show({
                            title: "Name Taken",
                            message: "This name is already taken",
                            color: "red",
                            autoClose: 2000
                        })
                        setName("")
                        setLoading(false)
                        subscription.unsubscribe()
                    }
                })
                session.socket.send(name)
            }
        }}>
            Set Name
        </Button>
    </Modal>



}