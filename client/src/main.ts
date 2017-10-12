import * as _ from 'lodash';
import * as Phaser from 'phaser-ce';
import User from './objects/user';
import Bullet, {Hit} from './objects/bullet';
import GameData from './game_state';
import EventEmitter from 'wolfy87-eventemitter';
import * as State from './states';

import * as ReconnectingWebsocket from 'reconnecting-websocket';

type Point = Phaser.Point;
const Point = Phaser.Point;

export default class Main extends Phaser.Game {
    ws: ReconnectingWebsocket;
    objects: any;
    data: GameData;
    ee: EventEmitter;

    constructor() {
        super(800, 600, Phaser.AUTO, 'content');
        //super(800, 600, Phaser.AUTO, 'content', {
            //init() {
                //this.physics.startSystem(Phaser.Physics.ARCADE);
            //},
            //preload: preload,
            //create: this.start.bind(this),
        //});

        this.data = new GameData(this);
        this.ee = new EventEmitter<string>();
        this.objects = {};
        this.state.add('boot', new State.Boot(this));
        this.state.add('start', new State.Start(this));
        this.state.start('boot');
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

        this.ee.on('SyncGameData', (data) => {
            this.state.syncWith(data);
        });

    }

    startFire() {
        return new Promise((resolve, reject) => {
            this.input.onDown.addOnce(() => {
                const promise = new Promise((resolve, reject) => {
                    const me = this.data.me();
                    me.startSpin();
                    this.input.onDown.addOnce(() => resolve(me));
                }).then((me: User) => {
                    const [pos, angle] = me.stopSpin();
                    this.send({
                        Fire: {
                            pos: pos,
                            angle: angle,
                        }
                    });
                    resolve();
                });
            }, this);
        });
    }
}
