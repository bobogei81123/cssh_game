import Main from '../main';
import * as Phaser from 'phaser-ce';
import GameData from '../game_data';

export class Boot extends Phaser.State {
    constructor(public main: Main) {
        super();
    }

    preload() {
        this.load.baseURL = 'assets/';
        this.load.image('background_far', 'background/farback.gif');
        this.load.image('background_near', 'background/starfield.png');

        this.load.spritesheet('explosion', 'explosion.png', 64, 64);

        this.load.image('ship-ally', 'kenny/playerShip1_blue.png');
        this.load.image('ship-enemy', 'kenny/playerShip2_red.png');

        this.load.image('bullet10', 'bullet10.png');
        this.load.image('check', 'check.png');
        this.load.image('cross', 'cross.png');
        this.load.image('button', 'button.png');
        this.load.image('button-dark', 'button-dark.png');

        this.load.image('win', 'misc/win.png');
        this.load.image('lose', 'misc/lose.png');
        this.load.image('dead', 'misc/dead.png');

        this.load.atlasXML('ui',
            'uipack-space/Spritesheet/uipackSpace_sheet.png',
            'uipack-space/Spritesheet/uipackSpace_sheet.xml',)

        this.load.image('vs', 'misc/vs.png');
    }

    create() {
        this.makeView();
        this.game.stage.disableVisibilityChange = true;
        this.game.time.advancedTiming = true;
        this.initialize();
    }

    makeView() {
        let backgroundFar = this.game.add.tileSprite(0, 0, this.game.width, this.game.height, 'background_far');
        backgroundFar.autoScroll(-3, 0);
        let backgroundNear = this.game.add.tileSprite(0, 0, this.game.width, this.game.height, 'background_near');
        backgroundNear.autoScroll(-15, 0);

        this.game.scale.scaleMode = Phaser.ScaleManager.SHOW_ALL;
        this.game.scale.refresh();

        window.addEventListener('resize', () => {
            this.game.scale.refresh();
        });

        this.main.objects.ping_info_text =
            this.game.add.text(760, 10, 'Ping: ', {fill: '#FFFFFF', fontSize: 10});
    }

    async enterName(): Promise<any> {
        const $enterNameModal = document.getElementById('enter-name-modal');
        const $submitButton = document.getElementById('submit-button');
        $enterNameModal.classList.add('is-active');
        return new Promise((resolve, reject) => {
            $submitButton.onclick = () => {
                const $nameInput = <HTMLInputElement>document.getElementById('name-input');
                $submitButton.onclick = null;
                $enterNameModal.classList.remove('is-active');
                resolve($nameInput.value);
            };
        });
    }

    async initialize() {
        this.main.data = new GameData(this.main);
        await this.main.connectWebsocket();

        this.main.ee.on('ping', (data) => {
            this.main.objects.ping_info_text.text = `Ping: ${data}\nFPS: ${this.game.time.fps}`;
        });
        this.main.send('RequestInitial');

        const initial = await this.main.waitForEvent('Initial');
        this.main.data.id = initial.id;

        const name = await this.enterName();
        this.game.state.start('room', false, false, name);
    }
}
