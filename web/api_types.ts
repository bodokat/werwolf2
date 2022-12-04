export type ToClient =
  | {
      joined: Player;
    }
  | {
      left: Player;
    }
  | {
      message: {
        content: string;
        from: Player;
        [k: string]: unknown;
      };
    };

export interface Player {
  name: string;
  [k: string]: unknown;
}

export type ToServer = {
  message: {
    content: string;
    from: Player;
    [k: string]: unknown;
  };
};