import { Button, Group, List, Paper, Stack, Text } from "@mantine/core";
import { ToClient } from "../message";
import { sessionContext, stateContext } from "./index";
import React, { Fragment, ReactElement, useContext, useState } from "react";

export function Messages() {
    let messages = useContext(stateContext)!.messages
    return <List listStyleType="none" spacing={"lg"} >
        {messages.map((m, index) => RenderMessage({ message: m }))
            .filter((e): e is React.ReactElement => e != null)
            .map((e, index) => <List.Item key={index}>{e}</List.Item>)}
    </List>
}

function RenderMessage({ message }: { message: ToClient }): React.ReactElement | null {
    switch (message.type) {
        case "welcome":
            console.warn("received initial message again")
            return null
        case "new_settings":
            return null
        case "joined":
            return <Paper>{message.player.name} joined</Paper>
        case "left":
            return <Paper>{message.player.name} left</Paper>
        case "started":
            return <Paper>Game started</Paper>
        case "ended":
            return <Paper>Game ended</Paper>
        case "name_accepted":
            return null
        case "name_rejected":
            return null
        case "text":
            return <Paper>{message.text}</Paper>
        case "question":
            return <QuestionMessage message={message} />
    }
}

function QuestionMessage({ message }: { message: ToClient & { type: "question" } }) {
    let session = useContext(sessionContext)!
    let [choice, setChoice] = useState<number>()
    return <Fragment>
        <Text>{message.text}</Text>
        {(choice == undefined ?
            <Group> {message.options.map((option, index) => {
                return <Button variant="filled" key={index}
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
            <Group><Button disabled variant="filled">{message.options[choice]}</Button> </Group>
        )}
    </Fragment>
}