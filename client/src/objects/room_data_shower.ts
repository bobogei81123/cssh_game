import * as Phaser from 'phaser-ce';
import {GAME} from '../constant';

class UserItem extends Phaser.Group {
    constructor(game: Phaser.Game, group: number, name: string) {
        super(game);
        const plane = new Phaser.Sprite(game, 0, 12, group ? 'ship-enemy' : 'ship-ally');
        plane.width = 40;
        plane.height = 40;
        plane.anchor.set(0.5, 1);
        this.addChild(plane);

        const text = new Phaser.Text(game, 0, 20, name, {fontSize: 20, fill: '#FFFFFF'});
        text.anchor.set(0.5, 0);
        this.addChild(text);
    }
}

export default class RoomDataShower extends Phaser.Group {
    user_items: UserItem[];

    constructor(game: Phaser.Game) {
        super(game);
        this.user_items = [];
        this.drawGlass();
    }

    drawGlass(): void {
        const _glass = new Phaser.Graphics(this.game);
        _glass.lineStyle(8, 0x60C0FF, 0.4);
        _glass.beginFill(0x80B0B0, 0.2);
        _glass.drawRoundedRect(0, 0, 600, 400, 10);

        _glass.lineStyle(3, 0x70E0FF, 0.5);
        _glass.moveTo(10, 200);
        _glass.lineTo(590, 200);
        const glass = new Phaser.Sprite(this.game, 100, 100, _glass.generateTexture());
        this.addChild(glass);

        const vs = new Phaser.Sprite(this.game, 400, 300, 'vs');
        vs.anchor.set(0.5);
        vs.width = 100;
        vs.height = 100;
        this.addChild(vs);
    }

    updateWithData({users, teams}) {

        for (let user of this.user_items) {
            user.destroy();
        }
        this.user_items = [];

        //this.clear();
        //this.lineStyle(5, 0xFFFFFF);
        //this.drawRoundedRect(80, 80, 640, 360, 30);
        //this.lineStyle(3, 0xFFFFFF);
        //this.moveTo(400, 100);
        //this.lineTo(400, 420);
        //console.log(users);

        for (let i=0, x=180; i<teams[0].length; i++, x+=100) {
            const userItem = new UserItem(this.game, 0, users[teams[0][i]].name);
            userItem.position.set(x, 180);
            this.addChild(userItem);
        }

        for (let i=0, x=180; i<teams[1].length; i++, x+=100) {
            const userItem = new UserItem(this.game, 1, users[teams[1][i]].name);
            userItem.position.set(x, 420);
            this.addChild(userItem);
        }

        //for (let i=0, y=100; i<teams[1].length; i++, y+=70) {
            //this.drawRoundedRect(420, y, 280, 50, 10);
            //this.addChild(new Phaser.Text(this.game, 440, y+5,
                //users[teams[1][i]].name, {fill: 'white'}));
        //}
    }
}
