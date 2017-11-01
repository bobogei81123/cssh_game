import * as Phaser from 'phaser-ce';
import {USER} from '../constant';

type Point = Phaser.Point;
const Point = Phaser.Point;

export class Hit {
    constructor(
        public target_point: Point,
        public onHit: () => void,
    ) {}
}

export default class Bullet extends Phaser.Sprite {

    constructor(game: Phaser.Game, public hit?: Hit) {
        super(game, 0, 0, 'bullet10');
        this.game.physics.arcade.enable(this);
        this.scale.set(0.8);

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
            .velocityFromAngle(angle * 180 / Math.PI, 400.0, this.body.velocity);
    }

    update() {
        if (this.hit != null) {
            let dx = this.hit.target_point.x - this.position.x;
            let dy = this.hit.target_point.y - this.position.y;
            let dis = Math.sqrt(dx * dx + dy * dy);

            if (dis < USER.RADIUS * 1.1) {
                this.hit.onHit();
                const explosion = this.game.add.sprite(
                    this.position.x, this.position.y, 'explosion');
                explosion.scale.set(0.7);
                explosion.anchor.set(0.5);
                explosion.animations.add('boom');
                explosion.play('boom', 20, false, true);
                this.destroy();
            }
        }
    }
}
