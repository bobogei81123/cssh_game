import * as Phaser from 'phaser-ce';
import {GAME} from '../constant';

export default class RoomDataShower extends Phaser.Graphics {
    constructor(game: Phaser.Game) {
        super(game);
    }

    updateWithData({users, teams}) {
        this.clear();
        this.lineStyle(5, 0xFFFFFF);
        this.drawRoundedRect(80, 80, 640, 360, 30);
        this.lineStyle(3, 0xFFFFFF);
        this.moveTo(400, 100);
        this.lineTo(400, 420);
        console.log(users);

        for (let i=0, y=100; i<teams[0].length; i++, y+=70) {
            this.drawRoundedRect(100, y, 280, 50, 10);
            this.addChild(new Phaser.Text(this.game, 120, y+5,
                users[teams[0][i]].name, {fill: 'white'}));
        }

        for (let i=0, y=100; i<teams[1].length; i++, y+=70) {
            this.drawRoundedRect(420, y, 280, 50, 10);
            this.addChild(new Phaser.Text(this.game, 440, y+5,
                users[teams[1][i]].name, {fill: 'white'}));
        }
    }
}
