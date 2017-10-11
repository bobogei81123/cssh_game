import * as Phaser from 'phaser-ce';
type Point = Phaser.Point;
const Point = Phaser.Point;

export default class User extends Phaser.Sprite {
    main: Phaser.Sprite;
    healthBar: Phaser.Graphics;
    arrow: Phaser.Graphics;
    timer?: Phaser.Timer;
    loopEvent?: Phaser.TimerEvent;
    _lastTick?: number;
    //maxHealth: number;
    //health: number;

    static HEALTHBAR_WIDTH = 30;
    static HEALTHBAR_HEIGHT = 2;

    static HZ = 0.6;
    static UPDATE_TICK = 0.016;

    static RADIUS = 18;
    static BASE_RADIUS = 15;

    constructor(game: Phaser.Game, key: string, pos: Point, health=100, maxHealth=100) {
        super(game, pos.x, pos.y, key);
        this.texture.baseTexture.scaleMode = PIXI.scaleModes.NEAREST;
        this.scale.set(User.RADIUS / User.BASE_RADIUS);

        this.animations.add('stand');
        this.animations.play('stand', 10, true);
        this.anchor.set(0.5);

        this.health = health;
        this.maxHealth = maxHealth;

        const healthBar = new Phaser.Graphics(game, 0, -20);
        this.healthBar = healthBar;
        this.addChild(healthBar);

        const arrow = new Phaser.Graphics(this.game, 0, 0);
        arrow.beginFill(0x33aaff);
        arrow.drawTriangle([
            new Point(25, -4),
            new Point(25, 4),
            new Point(35, 0),
        ]);
        arrow.visible = false;
        this.arrow = arrow;
        this.addChild(arrow);

        //const circle = new Phaser.Graphics(this.game, 0, 0);
        //circle.beginFill(0xff0000);
        //circle.drawCircle(0, 0, User.RADIUS * 2);
        //this.addChild(circle);

        this.game.world.add(this);
    }

    update() {
        this.healthBar.clear();
        this.healthBar.beginFill(0xff0000);
        this.healthBar.drawRect(
            -User.HEALTHBAR_WIDTH/2,
            -User.HEALTHBAR_HEIGHT/2,
            User.HEALTHBAR_WIDTH,
            User.HEALTHBAR_HEIGHT
        );
        this.healthBar.beginFill(0x00ff00);
        let nw = User.HEALTHBAR_WIDTH * (this.maxHealth - this.health) / this.maxHealth;
        this.healthBar.drawRect(
            -User.HEALTHBAR_WIDTH/2 + nw,
            -User.HEALTHBAR_HEIGHT/2,
            User.HEALTHBAR_WIDTH - nw,
            User.HEALTHBAR_HEIGHT
        );
    }

    startSpin() {
        this.arrow.visible = true;
        this.arrow.rotation = Math.random() * Math.PI * 2;
        this.timer = new Phaser.Timer(this.game);
        this._lastTick = (new Date()).valueOf();
        this.loopEvent = this.timer.loop(User.UPDATE_TICK * 1000, () => {
            this.arrow.rotation += User.HZ * 2 * Math.PI * User.UPDATE_TICK;
            this._lastTick = (new Date()).valueOf();
        });
        this.game.time.add(this.timer);
        this.timer.start();
    }

    stopSpin() {
        const current = (new Date()).valueOf();
        const diff = current - this._lastTick;

        this.arrow.rotation += User.HZ * 2 * Math.PI * diff / 1000;
        this.arrow.visible = false;
        this.timer.destroy();
        return [{x: this.position.x, y: this.position.y}, this.arrow.rotation];
    }
}
