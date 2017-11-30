import * as Phaser from 'phaser-ce';
import {GAME} from '../constant';
import {RoomData} from '../server_data/room';

class UserItem extends Phaser.Group {
    constructor(
        game: Phaser.Game,
        is_enemy: boolean,
        name: string,
        public ready: boolean
    ) {
        super(game);
        const plane = new Phaser.Sprite(game, 0, 12, 'space',
            is_enemy ? 'playerShip2_red.png' : 'playerShip1_blue.png');
        plane.width = 60;
        plane.height = 60;
        plane.anchor.set(0.5);
        this.addChild(plane);

        const text = new Phaser.Text(game, 0, 54, name, {fontSize: 12, fill: '#FFFFFF'});
        text.anchor.set(0.5);
        this.addChild(text);

        if (ready) {
            const shield = new Phaser.Sprite(game, 0, 12, 'space', 'shield2.png');
            shield.anchor.set(0.5);
            shield.scale.set(0.7);
            this.addChild(shield);
        }
    }
}

export default class RoomDataShower extends Phaser.Group {
    constructor(game: Phaser.Game) {
        super(game);
        //this.user_items = [];
        this.drawGlass();
    }

    drawGlass(): void {
        const room = new Phaser.Sprite(this.game, 0, 0, 'room');
        this.addChild(room);
    }

    updateWithData({players, teams}: RoomData) {
        this.removeBetween(1);
        for (let t of [0, 1]) {
            for (let [i, id] of teams[t].entries()) {
                const x = (t == 0 ? 175 : 275) + 100 * i;
                const {name, ready} = players[id];
                const userItem = new UserItem(this.game, t == 1, name, ready);
                userItem.position.set(x, t == 0 ? 150 : 350);
               this.addChild(userItem);
            }
        }
    }
}
