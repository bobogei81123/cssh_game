import Main from '../main';
import * as Phaser from 'phaser-ce';

export class Boot extends Phaser.State {
    constructor(public main: Main) {
        super();
    }

    preload() {
        this.load.image('background_far', 'assets/background/farback.gif');
        this.load.image('background_near', 'assets/background/starfield.png');

        this.load.spritesheet('explosion', 'assets/explosion.png', 64, 64);

        for (let i=1; i<=4; i++) {
            this.load.spritesheet(`UFO${i}`, `assets/UFO_40x30/${i}.png`, 40, 30, 4);
        }

        this.load.image('bullet10', 'assets/bullet10.png')
    }

    create() {
        this.makeView();
        this.game.time.advancedTiming = true;
        this.initialize();
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

        this.main.objects.ping_info_text =
            this.game.add.text(760, 10, 'Ping: ', {fill: '#FFFFFF', fontSize: 10});
    }

    initialize() {
        const getInit = new Promise((resolve, reject) => {
            this.main.ee.once('Initial', (data) => {
                this.main.data.my_id = data.your_id;
                resolve();
            });
        });
        const getStateSync = new Promise((resolve, reject) => {
            this.main.ee.once('SyncGameState', (data) => {
                this.main.data.syncWith(data);
                resolve();
            });
        });

        this.main.connectWebsocket();
        this.main.ws.onopen = () => {
            this.main.send('Join');
            this.main.ws.onopen = null;
        }
        let combined = Promise.all([getInit, getStateSync]);
        combined.then(() => this.game.state.start('start', false));

        this.main.ee.on('ping', (data) => {
            this.main.objects.ping_info_text.text = `Ping: ${data}\nFPS: ${this.game.time.fps}`;
        });
    }
}
