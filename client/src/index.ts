require('pixi');
require('p2');
require('phaser');

import * as ReconnectingWebsocket from 'reconnecting-websocket';

class SimpleGame {
    game: Phaser.Game;
    ws: ReconnectingWebsocket;

    constructor() {
        this.game = new Phaser.Game(800, 600, Phaser.AUTO, 'content', {
            preload: this.preload,
            create: this.create,
        });
        this.connectWebsocket();
    }

    preload() {
        console.log("Hao123");
    }

    create() {
        this.game.add.text(100, 100, 'Hao123', {fill: '#FF0000'});
    }

    connectWebsocket() {
        this.ws = new ReconnectingWebsocket(
            `ws://${window.location.hostname}:3210`, ['rust-websocket']);
        this.ws.onmessage = console.log;
    }
}

window.onload = () => {
    const game = new SimpleGame();
}
