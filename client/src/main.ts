import * as _ from 'lodash';
import * as Phaser from 'phaser-ce';
import User from './objects/user';
import Bullet, {Hit} from './objects/bullet';
import GameData from './game_data';
import EventEmitter from 'wolfy87-eventemitter';
import * as State from './states';
import {GAME} from './constant';

//import * as ReconnectingWebsocket from 'reconnecting-websocket';

type Point = Phaser.Point;
const Point = Phaser.Point;

export default class Main extends Phaser.Game {
    ws: WebSocket;
    objects: any;
    data: GameData;
    ee: EventEmitter;

    constructor() {
        super(GAME.WIDTH, GAME.HEIGHT, Phaser.AUTO, 'content');

        this.data = new GameData(this);
        this.ee = new EventEmitter<string>();
        this.objects = {};
        this.state.add('boot', new State.Boot(this));
        this.state.add('room', new State.Room(this));
        this.state.add('start', new State.Start(this));
        this.state.start('boot');

    }

    connectWebsocket(): Promise<any> {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(
                `ws://${window.location.hostname}:3210`, ['rust-websocket']);

            this.ws.onopen = () => {
                this.ws.onopen = null;
                resolve();
            };

            this.ws.onmessage = (e) => {
                if ('data' in e) {
                    this.receive(e.data);
                }
            };
        });
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
        if (typeof parsed == 'string') {
            this.ee.emitEvent(parsed);
            console.log(parsed);
            return;
        }
        const ks = Object.keys(parsed);
        if (!ks.length) return;

        if (ks[0] != 'ping') console.log(parsed);

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

    waitForEvent(name): Promise<any> {
        return new Promise((resolve, reject) => {
            this.ee.once(name, (data) => {
                resolve(data)
            });
        });
    }

}
