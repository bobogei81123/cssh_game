import User from './objects/user';
import * as _ from 'lodash';
import {PlayersData, Player as PlayerData} from './server_data/start';

export default class GameData {
    id: number;
    team: number;
    players: { [index: number]: User; };

    constructor(public game) {
        this.players = {};
    }

    me(): User {
        return this.players[this.id];
    }

    addUser(data: PlayerData) {
        const id = data.id;
        if (id in this.players) {
            this.removeUser(id);
        }
        this.players[id] = new User(this.game, data, this.team == data.team); 
    }

    removeUser(id: number) {
        if (id in this.players) {
            this.players[id].destroy();
            delete this.players[id];
        }
    }

    syncUser(id: number, data: PlayerData) {
        if (!(id in this.players)) return;

        const user = this.players[id];
        user.syncWith(data);
    }

    syncWith(players: PlayersData) {
        this.team = players[this.id].team;
        const shouldRemove = _.difference(_.keys(this.players), _.keys(players));
        const shouldAdd = _.difference(_.keys(players), _.keys(this.players));
        const shouldSync = _.intersection(_.keys(players), _.keys(this.players));

        for (let id of shouldRemove) {
            this.removeUser(id as any);
        }

        for (let id of shouldAdd) {
            this.addUser(players[id]);
        }

        for (let id of shouldSync) {
            this.syncUser(id as any, players[id]);
        }
    }

    setHealth(id, val) {
        this.players[id].health.set(val);
    }
}
