import User from './objects/user';
import * as _ from 'lodash';

export default class GameData {
    id: number;
    players: { [index: number]: User; };

    constructor(public game) {
        this.players = {};
    }

    me(): User {
        return this.players[this.id];
    }

    addUser(data) {
        const id = data.id;
        if (id in this.players) {
            this.removeUser(id);
        }
        this.players[id] = new User(this.game, 'UFO1', data.pos, data.health.value, data.health.max); 
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
        user.position.x = data.pos.x;
        user.position.y = data.pos.y;
        user.health = data.health.value;
        user.maxHealth = data.health.max;
    }

    syncWith(data) {
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
        this.players[id].health = val;
    }
}
