import { Button, Group, List, Paper, Stack, Text } from "@mantine/core";
import { ToClient } from "../../../bindings/ToClient";
import { sessionContext, stateContext } from "./index";
import React, { Fragment, useContext, useState } from "react";

export function Messages() {
    let messages = useContext(stateContext)!.messages
    return <Stack justify="flex-end" h={300}>
        {messages.map((m, index) => <Paper key={index}>{renderMessage(m)}</Paper>)}
    </Stack>
}

function renderMessage(message: ToClient): React.ReactNode {
    switch (message.type) {
        case "welcome":
            console.warn("received initial message again")
            return
        case "newsettings":
            return
        case "joined":
            return `${message.player.name} joined`
        case "left":
            return `${message.player.name} left`
        case "started":
            return "Game started"
        case "ended":
            return "Game ended"
        case "nameaccepted":
            return
        case "namerejected":
            return
        case "text":
            return message.text
        case "question":
            return <QuestionMessage message={message} />
    }
}

function QuestionMessage({ message }: { message: ToClient & { type: "question" } }) {
    let session = useContext(sessionContext)!
    let [choice, setChoice] = useState<number>()
    return <Fragment key={message.id}>
        <Text>{message.text}</Text>
        {(choice == undefined ?
            <Group> {message.options.map((option, index) => {
                return <Button variant="filled"
                    onClick={() => {
                        setChoice(index)
                        session.send({
                            type: "response",
                            id: message.id,
                            choice: index

                        })
                    }}>
                    {option}
                </Button>
            })} </Group> :
            <Button disabled variant="filled">{message.options[choice]}</Button>
        )}
    </Fragment>
}