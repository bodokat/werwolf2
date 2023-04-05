export class ChatSession {
    constructor({ lobby, name }: { lobby: string, name: string }) {

    }
}

function create_chat_session(lobby: string, name: string): WebSocket {
    const protocol = (document.location.hostname in ["0.0.0.0", "localhost"]) ? "ws" : "wss"
    const url = `${protocol}://${document.location.host}/api/chat/${lobby}?name=${name}`

    return new WebSocket(url)
}