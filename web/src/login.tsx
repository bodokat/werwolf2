import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@mantine/core";
import { TextInput } from "@mantine/core";
import { Stack } from "@mantine/core";
import React from "react";
import { GameSession, newLobby } from "./api/gameSession";

export function Login() {
    let [loading, setLoading] = useState(false)
    let navigate = useNavigate()
    return <Stack>
        <Button loading={loading} variant="filled" onClick={async () => {
            setLoading(true)
            let lobby = await newLobby()
            navigate(`/l/${lobby}`)
        }}>
            New Lobby
        </Button>
    </Stack>
}