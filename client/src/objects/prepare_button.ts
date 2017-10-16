import * as Phaser from 'phaser-ce';

export default class PrepareButton extends Phaser.Button {
    text: Phaser.Text;

    constructor(game: Phaser.Game, x: number, y: number) {
        super(game, x, y, 'button-dark');
        this.anchor.set(0.5);
        this.scale.set(0.5);
        this.text = new Phaser.Text(game, 0, 0, '準備好了', {fill: 'white', align: 'center', fontSize: 35});
        this.text.anchor.set(0.5);
        this.text.scale.set(2.0);
        this.addChild(this.text);
        this.onInputUp.addOnce(() => {
            this.loadTexture('button');
            this.text.text = '等待其他玩家';
        });
    }
}
