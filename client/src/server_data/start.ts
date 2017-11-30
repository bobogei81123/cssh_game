export interface Player {
    id: number, 
    name: string,
    team: number,
    pos: Point,
    health: Health,
}

export interface Point {
    x: number,
    y: number,
}

export interface Health {
    value: number,
    max: number,
}

export type PlayersData = {[_: number]: Player};
