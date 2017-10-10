require('pixi');
require('p2');
require('phaser');
import * as Phaser from 'phaser-ce';

type Point = Phaser.Point;
const Point = Phaser.Point;

import * as ReconnectingWebsocket from 'reconnecting-websocket';

class Bullet extends Phaser.Sprite {
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
        this.game.physics.arcade.velocityFromAngle(angle * 180 / Math.PI, 300.0, this.body.velocity);
    }

    update() {
    }
}

class User extends Phaser.Sprite {
    main: Phaser.Sprite;
    healthBar: Phaser.Graphics;
    //maxHealth: number;
    //health: number;

    static HEALTHBAR_WIDTH = 30;
    static HEALTHBAR_HEIGHT = 2;

    constructor(game: Phaser.Game, key: string, pos: Point) {
        super(game, pos.x, pos.y, key);
        this.texture.baseTexture.scaleMode = PIXI.scaleModes.NEAREST;
        this.animations.add('stand');
        this.animations.play('stand', 10, true);
        this.anchor.set(0.5);

        this.maxHealth = 100;
        this.health = 100;

        let graphic = new Phaser.Graphics(game, 0, -20);
        this.healthBar = graphic;

        this.addChild(graphic);
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
}

class Receiver {
    constructor(public main: Main) {
    }

    ping(val: number) {
        this.main.ping_info_text.text = `Ping: ${val}`;
    }

    GameStateInfo(data) {
        const main = this.main;
        const state = main.game_state;
        state.started = true;
        state.my_id = data.your_id;

        for (let id in state.users) {
            state.users[id].destroy();
        }
        state.users = {};

        for (let user of data.users) {
            state.users[user.id] = new User(main.game, 'UFO1', new Point(user.pos.x, user.pos.y))
        }
    }

    Fire([id, data]) {
        const main = this.main;
        const bullet = new Bullet(main.game, 'bullet10');
        bullet.fire(data.pos, data.angle);
    }
}

class GameState {
    started: boolean;
    my_id?: number;
    users: { [index: number]: any; };
    spinning: boolean;

    constructor() {
        this.started = false;
        this.users = {};
        this.spinning = false;
    }

    me(): any {
        return this.users[this.my_id];
    }
}

class Main {
    game: Phaser.Game;
    ws: ReconnectingWebsocket;
    ping_info_text: Phaser.Text;
    game_state: GameState;
    users: Array<User>;
    receiver: Receiver;

    constructor() {
        this.game = new Phaser.Game(800, 600, Phaser.AUTO, 'content', {
            init() {
                this.physics.startSystem(Phaser.Physics.ARCADE);
            },
            preload() {
                this.load.image('background_far', 'assets/background/farback.gif');
                this.load.image('background_near', 'assets/background/starfield.png');

                for (let i=1; i<=4; i++) {
                    this.load.spritesheet(`UFO${i}`, `assets/UFO_40x30/${i}.png`, 40, 30, 4);
                }

                this.load.image('bullet10', 'assets/bullet10.png')
            },
            create: this.create.bind(this),
        });

        this.receiver = new Receiver(this);
        this.game_state = new GameState();
        //this.game.input.onDown.addOnce(this.startSpin, this);
    }

    create() {
        this.game.add.sprite(0, 0, 'background_far');
        let backgroundNear = this.game.add.tileSprite(0, 0, this.game.width, this.game.height, 'background_near');
        backgroundNear.autoScroll(-20, 0);

        this.game.scale.scaleMode = Phaser.ScaleManager.SHOW_ALL;
        this.game.scale.refresh();

        this.ping_info_text = this.game.add.text(760, 10, 'Ping: ', {fill: '#FFFFFF', fontSize: 10});
        this.connectWebsocket();


        window.addEventListener('resize', () => {
            this.game.scale.refresh();
        });

        setTimeout(this.startGame.bind(this), 1000);

        this.game.input.onDown.addOnce(this.startSpin, this);
    }

    connectWebsocket() {
        this.ws = new ReconnectingWebsocket(
            `ws://${window.location.hostname}:3210`, ['rust-websocket']);

        this.ws.onmessage = (e) => {
            if ('data' in e) {
                this.receive(e.data);
            }
        }
    }

    send(data) {
        this.ws.send(JSON.stringify(data));
    }

    receive(data) {
        let parsed;
        try {
            parsed = JSON.parse(data);
        } catch (err) {
            console.log(`Parse data failed: ${data}`);
        }

        this._receive(parsed);
    }

    _receive(parsed) {
        console.log(parsed);
        const ks = Object.keys(parsed);
        if (!ks.length) return;

        const key = ks[0];

        if (!(key in this.receiver)) return;

        this.receiver[key](parsed[key]);
    }

    startGame() {
        this.send("Join");
    }

    startSpin() {
        if (!this.game_state.started) {
            this.game.input.onDown.addOnce(this.startSpin, this);
            return;
        }

        const promise = new Promise((resolve, reject) => {
            this._startSpin();
            this.game.input.onDown.addOnce(resolve);
        }).then(() => {
            const pos = this.game_state.me().worldPosition;
            this.send({
                Fire: {
                    pos: {
                        x: pos.x,
                        y: pos.y,
                    },
                    angle: this.arrow.rotation,
                }
            });
            this.arrow.destroy();
            this.game.input.onDown.addOnce(this.startSpin, this);
        });
    }

    _startSpin() {
        console.log('Spin');
        const timer = new Phaser.Timer(this.game);
        const pos = this.game_state.me().worldPosition;

        const arrow = new Phaser.Graphics(this.game, pos.x, pos.y);
        arrow.beginFill(0x1166ff);
        arrow.drawTriangle([
            new Point(25, -4),
            new Point(25, 4),
            new Point(35, 0),
        ]);

        this.game.world.add(arrow);

        timer.loop(20, () => {
            arrow.rotation += 0.1;
        });

        this.game.time.add(timer);
        timer.start();

        this.arrow = arrow;
    }
}

window.onload = () => {
    const main = new Main();
}

function applyMixins(derivedCtor: any, baseCtors: any[]) {
    baseCtors.forEach(baseCtor => {
        Object.getOwnPropertyNames(baseCtor.prototype).forEach(name => {
            derivedCtor.prototype[name] = baseCtor.prototype[name];
        });
    });
}
