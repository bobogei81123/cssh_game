require('pixi');
require('p2');
require('phaser');
import * as Phaser from 'phaser-ce';
import User from './object/user';
import Bullet from './object/bullet';
import GameState from './game_state';
import EventEmitter from 'wolfy87-eventemitter';

import * as ReconnectingWebsocket from 'reconnecting-websocket';

type Point = Phaser.Point;
const Point = Phaser.Point;


function preload() {
    this.load.image('background_far', 'assets/background/farback.gif');
    this.load.image('background_near', 'assets/background/starfield.png');

    for (let i=1; i<=4; i++) {
        this.load.spritesheet(`UFO${i}`, `assets/UFO_40x30/${i}.png`, 40, 30, 4);
    }

    this.load.image('bullet10', 'assets/bullet10.png')
}

class Main {
    game: Phaser.Game;
    ws: ReconnectingWebsocket;
    objects: any;
    state: GameState;
    ee: EventEmitter;

    constructor() {
        this.game = new Phaser.Game(800, 600, Phaser.AUTO, 'content', {
            init() {
                this.physics.startSystem(Phaser.Physics.ARCADE);
            },
            preload: preload,
            create: this.start.bind(this),
        });

        this.state = new GameState(this.game);
        this.ee = new EventEmitter<string>();
        this.objects = {};
    }

    start() {
        this.makeView();
        this.connectWebsocket();
        this.ws.onopen = () => this.send('Join');
        this.registEvents();
        this.game.time.advancedTiming = true;
    }

    makeView() {
        this.game.add.sprite(0, 0, 'background_far');
        let backgroundNear = this.game.add.tileSprite(0, 0, this.game.width, this.game.height, 'background_near');
        backgroundNear.autoScroll(-20, 0);

        this.game.scale.scaleMode = Phaser.ScaleManager.SHOW_ALL;
        this.game.scale.refresh();

        window.addEventListener('resize', () => {
            this.game.scale.refresh();
        });

        this.objects.ping_info_text =
            this.game.add.text(760, 10, 'Ping: ', {fill: '#FFFFFF', fontSize: 10});
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
        const ks = Object.keys(parsed);
        if (!ks.length) return;

        const key = ks[0];
        const data = parsed[key];
        if (data == null) {
            this.ee.emitEvent(key);
        } else if (!(data instanceof Array)) {
            this.ee.emitEvent(key, [data]);
        } else {
            this.ee.emitEvent(key, data);
        }
    }

    registEvents() {
        this.ee.on('ping', (data) => {
            this.objects.ping_info_text.text = `Ping: ${data}\nFPS: ${this.game.time.fps}`;
        });

        this.ee.on('GameStateInfo', (data) => {
            this.state.my_id = data.your_id;
            for (let user of data.users) {
                this.state.addUser(user);
            }
            this.startGame();
        });

        this.ee.on('Fire', (id, data) => {
            const bullet = new Bullet(this.game, 'bullet10');
            bullet.fire(new Point(data.pos.x, data.pos.y), data.angle);
        });
    }

    startGame() {
        this.game.input.onDown.addOnce(this.startSpin, this);
    }

    startSpin() {
        const promise = new Promise((resolve, reject) => {
            const me = this.state.me();
            me.startSpin();
            this.game.input.onDown.addOnce(() => resolve(me));
        }).then((me: User) => {
            const [pos, angle] = me.stopSpin();
            this.send({
                Fire: {
                    pos: pos,
                    angle: angle,
                }
            });
            this.game.input.onDown.addOnce(this.startSpin, this);
        });
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
