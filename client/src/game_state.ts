import User from './object/user';
import * as _ from 'lodash';

export default class GameState {
    my_id: number;
    users: { [index: number]: User; };

    constructor(public game) {
        this.users = {};
    }

    me(): User {
        return this.users[this.my_id];
    }

    addUser(data) {
        const id = data.id;
        if (id in this.users) {
            this.removeUser(id);
        }
        this.users[id] = new User(this.game, 'UFO1', data.pos, data.health.value, data.health.max); 
    }

    removeUser(id) {
        if (id in this.users) {
            this.users[id].destroy();
            delete this.users[id];
        }
    }

    syncUser(id, data) {
        if (!(id in this.users)) return;

        const user = this.users[id];
        user.position.x = data.pos.x;
        user.position.y = data.pos.y;
        user.health = data.health.value;
        user.maxHealth = data.health.max;
    }

    syncWith(data) {
        const shouldRemove = _.difference(_.keys(this.users), _.keys(data.users));
        const shouldAdd = _.difference(_.keys(data.users), _.keys(this.users));
        const shouldSync = _.intersection(_.keys(data.users), _.keys(this.users));

        for (let id of shouldRemove) {
            this.removeUser(id);
        }

        for (let id of shouldAdd) {
            this.addUser(data.users[id]);
        }

        for (let id of shouldSync) {
            this.syncUser(id, data.users[id]);
        }
    }

    setHealth(id, val) {
        this.users[id].health = val;
    }
}
