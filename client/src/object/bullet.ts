import * as Phaser from 'phaser-ce';
type Point = Phaser.Point;
const Point = Phaser.Point;

export default class Bullet extends Phaser.Sprite {
    constructor(game: Phaser.Game, key: string) {
        super(game, 0, 0, key);
        this.game.physics.arcade.enable(this);
        this.texture.baseTexture.scaleMode = PIXI.scaleModes.NEAREST;

        this.anchor.set(0.5);

        this.checkWorldBounds = true;
        this.outOfBoundsKill = true;
        this.exists = false;
    }

    fire(from: Point, angle: number) {
        this.game.world.add(this);
        this.reset(from.x, from.y);
        this.scale.set(1);
        this.rotation = angle;
        this.game.physics.arcade
            .velocityFromAngle(angle * 180 / Math.PI, 300.0, this.body.velocity);
    }
}
