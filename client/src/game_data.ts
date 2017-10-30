import User from './objects/user';
import * as _ from 'lodash';

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

    addUser(data) {
        console.log(data);
        const id = data.id;
        if (id in this.players) {
            this.removeUser(id);
        }
        this.players[id] = new User(this.game, data, this.team == data.team); 
    }

    removeUser(id) {
        if (id in this.players) {
            this.players[id].destroy();
            delete this.players[id];
        }
    }

    syncUser(id, data) {
        if (!(id in this.players)) return;

        const user = this.players[id];
        user.syncWith(data);
    }

    syncWith(data) {
        console.log(this);
        const shouldRemove = _.difference(_.keys(this.players), _.keys(data.players));
        const shouldAdd = _.difference(_.keys(data.players), _.keys(this.players));
        const shouldSync = _.intersection(_.keys(data.players), _.keys(this.players));

        for (let id of shouldRemove) {
            this.removeUser(id);
        }

        for (let id of shouldAdd) {
            this.addUser(data.players[id]);
        }

        for (let id of shouldSync) {
            this.syncUser(id, data.players[id]);
        }
    }

    setHealth(id, val) {
        this.players[id].health.set(val);
    }
}
