import User from './object/user'

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
}
