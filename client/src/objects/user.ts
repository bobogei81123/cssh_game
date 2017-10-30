import * as Phaser from 'phaser-ce';
import {USER} from '../constant';
type Point = Phaser.Point;
const Point = Phaser.Point;

class Health extends Phaser.Graphics {
    private health: number;
    private health_dirty: boolean;
    private max_health: number;

    constructor(game: Phaser.Game, x: number, y: number) {
        super(game, x, y);
        this.health_dirty = false;
    }

    set(h: number, h_max?: number) {
        this.health = h;
        if (h_max != null) this.max_health = h_max;
        this.health_dirty = true;
    }

    get(): number {
        return this.health;
    }

    update() {
        if (!this.health_dirty) return;

        this.clear();
        this.beginFill(0xff0000);
        this.drawRect(
            -USER.HEALTHBAR_WIDTH/2,
            -USER.HEALTHBAR_HEIGHT/2,
            USER.HEALTHBAR_WIDTH,
            USER.HEALTHBAR_HEIGHT
        );
        this.beginFill(0x00ff00);
        let nw = USER.HEALTHBAR_WIDTH * (this.max_health - this.health) / this.max_health;
        this.drawRect(
            -USER.HEALTHBAR_WIDTH/2 + nw,
            -USER.HEALTHBAR_HEIGHT/2,
            USER.HEALTHBAR_WIDTH - nw,
            USER.HEALTHBAR_HEIGHT
        );

        this.health_dirty = false;
    }
}

class Spinner extends Phaser.Sprite {
    
    constructor(game: Phaser.Game, x: number, y: number) {
        const arrow = new Phaser.Graphics(game, 0, 0);
        arrow.beginFill(0xffff33);
        arrow.drawTriangle([
            new Point(0, -5),
            new Point(0, 5),
            new Point(10, 0),
        ]);
        super(game, 0, 0, arrow.generateTexture());
        this.visible = false;
    }

    async spin(click_promise: Promise<any>) {
        this.visible = true;
        this.rotation = Math.random() * Math.PI * 2;
        const timer = new Phaser.Timer(this.game);
        let last_tick = Date.now();
        const loop_event = timer.loop(USER.UPDATE_TICK * 1000, () => {
            const now_tick = Date.now();
            const delta = now_tick - last_tick;
            this.rotation += USER.HZ * 2 * Math.PI * delta / 1000;
            last_tick = now_tick;
        });
        this.game.time.add(timer);
        timer.start();
        await click_promise;

        const now_tick = Date.now();
        const delta = now_tick - last_tick;

        this.rotation += USER.HZ * 2 * Math.PI * delta / 1000;
        this.visible = false;
        timer.destroy();

        return this.rotation;
    }
}

export default class User extends Phaser.Group {
    private body_sprite: Phaser.Sprite;
    private spinner: Spinner;
    private timer?: Phaser.Timer;
    private loop_event?: Phaser.TimerEvent;
    private _last_tick?: number;
    private halt_rotate_speed: number;

    name: string;
    team: number;
    friend: boolean;
    health: Health;

    constructor(game: Phaser.Game, data: any, friend: boolean) {
        const {name, team, pos, health} = data;
        super(game, null)
        this.friend = friend;
        this.health = new Health(this.game, 0, -USER.RADIUS-8);
        this.addChild(this.health);

        this.syncWith(data);

        this.body_sprite = new Phaser.Sprite(game, 0, 0, 'ship-' + (friend ? 'ally' : 'enemy'));
        this.body_sprite.width = USER.RADIUS * 2;
        this.body_sprite.height = USER.RADIUS * 2;
        this.body_sprite.anchor.set(0.5);
        this.addChild(this.body_sprite);


        this.spinner = new Spinner(game, 0, 0);
        this.spinner.pivot.set(-USER.RADIUS*2 + 10, 0);
        this.addChild(this.spinner);

        const name_text = new Phaser.Text(this.game, 0, USER.RADIUS+10, this.name,
            {fontSize: 10, fill: (this.friend ? '#99AAFF' : '#FF6666')});
        name_text.anchor.set(0.5);
        this.addChild(name_text);

        this.position = new Phaser.Point(pos.x, pos.y);
        this.halt_rotate_speed = this.game.rnd.sign() * this.game.rnd.realInRange(0.1, 0.2);
        this.game.world.add(this);
    }

    syncWith(data: any) {
        const {name, team, pos, health} = data;
        if (name != null) this.name = name;
        if (team != null) this.team = team;
        if (health != null) {
            this.health.set(health.value, health.max);
        }
        if (pos != null) this.position.set(pos.x, pos.y);
    }

    update() {
        super.update();
        this.body_sprite.rotation += this.halt_rotate_speed / 60;
    }

    async startSpin(click_promise: Promise<any>): Promise<[Point, number]> {
        const angle = await this.spinner.spin(click_promise);
        const pos = Point.add(this.position, (new Point).setToPolar(angle, USER.RADIUS));
        return [pos, angle]
    }

    rotateAndFire(angle: number): Promise<any> {
        this.body_sprite.rotation = this.body_sprite.rotation % (Math.PI*2);
        angle = (angle + Math.PI/2) % (Math.PI * 2);
        while (angle < this.body_sprite.rotation) angle += Math.PI*2;
        if (angle - this.body_sprite.rotation >= Math.PI) angle -= Math.PI*2;

        const tween = this.game.add.tween(this.body_sprite);
        tween.to({rotation: angle}, 500, Phaser.Easing.Quadratic.InOut);
        const promise = new Promise((resolve, reject) => {
            tween.onComplete.addOnce(() => {
                resolve();
            });
        });
        tween.start();
        return promise;
    }

    markDead() {
        const dead = new Phaser.Sprite(this.game, 0, 0, 'dead');
        dead.anchor.set(0.5);
        dead.scale.set(0.2);
        this.addChild(dead);
    }
}
