import { Button } from "@mantine/core";
import { TextInput } from "@mantine/core";
import { Stack } from "@mantine/core";
import React from "react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";

function Login() {
    const [name, setName] = useState("")
    const navigate = useNavigate()

    return (
        <Stack>
            <TextInput
                value={name}
                label="Enter name"
                onChange={(event) => setName(event.currentTarget.value)}
            />
            <Button
                onClick={() => { }}
            >
                Enter
            </Button>
        </Stack>
    )
}