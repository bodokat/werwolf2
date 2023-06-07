/* This file is generated and managed by tsync */

export type ToClient =
  | ToClient__Welcome
  | ToClient__NewSettings
  | ToClient__Joined
  | ToClient__Left
  | ToClient__Started
  | ToClient__NameAccepted
  | ToClient__NameRejected
  | ToClient__Text
  | ToClient__Question
  | ToClient__Ended;

type ToClient__Welcome = {
  type: "welcome";
  settings: LobbySettings;
  players: Array<string>;
};
type ToClient__NewSettings = {
  type: "new_settings";
  settings: LobbySettings;
};
type ToClient__Joined = {
  type: "joined";
  player: Player;
};
type ToClient__Left = {
  type: "left";
  player: Player;
};
type ToClient__Started = {
  type: "started";
};
type ToClient__NameAccepted = {
  type: "name_accepted";
  name: string;
};
type ToClient__NameRejected = {
  type: "name_rejected";
};
type ToClient__Text = {
  type: "text";
  text: string;
};
type ToClient__Question = {
  type: "question";
  id: number;
  text: string;
  options: Array<string>;
};
type ToClient__Ended = {
  type: "ended";
};

export type ToServer =
  | ToServer__Start
  | ToServer__Response
  | ToServer__Kick
  | ToServer__ChangeRoles;

type ToServer__Start = {
  type: "start";
};
type ToServer__Response = {
  type: "response";
  id: number;
  choice: number;
};
type ToServer__Kick = {
  type: "kick";
  player: Player;
};
type ToServer__ChangeRoles = {
  type: "change_roles";
  new_roles: Array<number>;
};

export interface Player {
  name: string;
}

export interface LobbySettings {
  available_roles: Array<string>;
  roles: Array<number>;
  admin?: string;
}
