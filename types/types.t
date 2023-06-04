choice ToClient {
    welcome: WelcomeMsg = 0
    new_settings: LobbySettings = 1
    joined: Player = 2
    left: Player = 3
    started = 4
    name_accepted: String = 5
    name_rejected = 6
    text: String = 7
    question: Question = 8
    ended = 9
}

choice ToServer {
    start = 0
    response: Response = 1
    kick: Player = 2
    change_roles: [U64] = 3
}

struct WelcomeMsg {
    settings: LobbySettings = 0
    players: [String] = 1
}

struct Question {
    id: U64 = 0
    text: String = 1
    options: [String] = 2
}

struct Response {
    id: U64 = 0
    selected: U64 = 1
}

struct Player {
    name: String = 0
}

struct LobbySettings {
    available_roles: [String] = 0
    roles: [U64] = 1
    optional admin: String = 2
}
