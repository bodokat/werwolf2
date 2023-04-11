import { LobbySettings } from "../../../bindings/LobbySettings";
import { ToServer } from "../../../bindings/ToServer";
import { ToClient } from "../../../bindings/ToClient";
import { BehaviorSubject, Observable, Subject, filter, firstValueFrom, map, scan, tap } from "rxjs";

type WelcomeMessage = ToClient & { type: "welcome" }

function isWelcomeMessage(msg: ToClient): msg is WelcomeMessage {
    return msg.type === "welcome"
}

function parseMessage(msg: string): ToClient {
    try {
        return JSON.parse(msg)
    } catch {
        console.error(`error parsing message: ${msg}`)
        throw new Error(`Unknown message: ${msg}`)

    }
}

export class GameSession {
    static async join(lobby: string): Promise<GameSession> {
        let url = join_lobby_url(lobby)
        console.log(`connecting to ${url}`)
        let socket = new WebSocket(url)
        let messages = new Subject<ToClient>()
        let handler = ({ data }: MessageEvent) => {
            return messages.next(parseMessage(data))
        }
        socket.addEventListener("message", handler)
        let initialMessage = await firstValueFrom(messages.pipe(filter(isWelcomeMessage)))
        let initialState = create_initial_state(initialMessage)
        return new GameSession({ socket, lobby, messages, initialState })
    }
    constructor({ socket, lobby, messages, initialState }: { socket: WebSocket, lobby: string, messages: Subject<ToClient>, initialState: GameState }) {
        this.socket = socket
        this.messages = messages

        let states = this.messages.pipe(scan(stateReducer, initialState))
        this.state = new BehaviorSubject(initialState)
        states.subscribe(this.state)
        this.messages.subscribe(new_message => console.log({ new_message }))
        this.state.subscribe(new_state => console.log({ new_state }))
    }

    send(message: ToServer) {
        this.socket.send(JSON.stringify(message))
    }

    socket: WebSocket

    messages: Subject<ToClient>

    state: BehaviorSubject<GameState>
}

function stateReducer(state: GameState, message: ToClient): GameState {
    let new_state = { ...state }
    new_state.messages.push(message)
    switch (message.type) {
        case "welcome":
            console.warn("Received additional welcome message")
            return new_state
        case "joined":
            new_state.players.push(message.player.name)
            return new_state
        case "left":
            new_state.players.push(message.player.name)
            return new_state
        case "nameaccepted":
            new_state.me = message.name
            return new_state
        case "namerejected":
            return new_state
        case "newsettings":
            new_state.settings = message
            return new_state
        case "started":
            new_state.started = true
            return new_state
        case "ended":
            new_state.started = false
            return new_state
        case "text":
            return new_state
        case "question":
            return new_state
    }
}

export async function newLobby(): Promise<string> {
    let response = await fetch(new_lobby_url, {
        method: "POST"
    })
    return await response.text()
}

const api_url: string = (
    import.meta.env.DEV ?
        `localhost:3000/api` :
        `${location.hostname}/api`
)

const new_lobby_url: URL = new URL(`${import.meta.env.DEV ? "http" : "https"}://${api_url}/new`)

const join_lobby_url = (lobby: string) => new URL(`${import.meta.env.DEV ? "ws" : "wss"}://${api_url}/join/${lobby}`)

export type GameState = {
    messages: ToClient[],
    players: string[],
    me: string | null,
    started: boolean,
    settings: LobbySettings
}

function create_initial_state(message: WelcomeMessage): GameState {
    return {
        messages: [],
        players: message.players,
        me: null,
        started: false,
        settings: message.settings
    }
}