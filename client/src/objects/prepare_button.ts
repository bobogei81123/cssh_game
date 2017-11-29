import * as Phaser from 'phaser-ce';


export default class PrepareButton extends Phaser.Button {
    //text: Phaser.Text;

    constructor(game: Phaser.Game, x: number, y: number) {
        super(game, x, y, 'ready-button');
        this.anchor.set(0.5);
        this.onInputUp.addOnce(() => {
            this.loadTexture('waiting-button');
        });
    }
}
