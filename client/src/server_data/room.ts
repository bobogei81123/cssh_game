export interface Player {
    id: number;
    name: string;
    team: number;
    ready: boolean;
}

export interface RoomData {
    players: {
        [_: number]: Player;
    };
    teams: [number[], number[]];
}

