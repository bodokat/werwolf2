import { useState } from "react";
import { useLocation, useNavigate, useParams } from "react-router-dom";
import { Stack, TextInput, Button } from "@mantine/core";

function Game() {
    const navigate = useNavigate()
    const location = useLocation()
    const params = useParams()

    const lobby = params["lobby"]
    if (!lobby) {
        return navigate("/")
    }

}

function ChooseName({ onSubmit }: { onSubmit: (name: string) => void }) {
    const [name, setName] = useState("")


    return (<Stack>
        <TextInput
            value={name}
            label="Enter name"
            onChange={(event) => setName(event.currentTarget.value)}
        />
        <Button
            onClick={() => onSubmit(name)}
        >
            Enter
        </Button>
    </Stack>)
}