import Main from '../main';
import * as Phaser from 'phaser-ce';
import GameData from '../game_data';

export class Boot extends Phaser.State {
    constructor(public main: Main) {
        super();
    }

    preload() {
        this.game.stage.disableVisibilityChange = true;
        this.game.time.advancedTiming = true;

        this.load.baseURL = 'assets/';
        //this.load.image('background_far', 'background/farback.gif');
        this.load.image('background_far', 'background/starfield2.jpg');
        //this.load.image('background_near', 'background/starfield.png');
        this.load.image('room', 'objects/room.png');

        this.load.spritesheet('explosion', 'effects/explosion.png', 64, 64);

        //this.load.image('ship-ally', 'kenny/playerShip1_blue.png');
        //this.load.image('ship-enemy', 'kenny/playerShip2_red.png');

        //this.load.image('bullet10', 'bullet10.png');
        this.load.image('check', 'check.png');
        this.load.image('cross', 'cross.png');
        this.load.image('ready-button', 'objects/ready-button.png');
        this.load.image('waiting-button', 'objects/waiting-button.png');

        this.load.image('win', 'misc/win.png');
        this.load.image('lose', 'misc/lose.png');
        //this.load.image('dead', 'misc/dead.png');
        this.load.image('refresh', 'objects/refresh-button.png');
        
        this.load.atlasXML('space',
            'spaceshooter/Spritesheet/sheet.png',
            'spaceshooter/Spritesheet/sheet.xml',)

        this.load.atlasXML('ui',
            'uipack-space/Spritesheet/uipackSpace_sheet.png',
            'uipack-space/Spritesheet/uipackSpace_sheet.xml',)

        //this.load.image('vs', 'misc/vs.png');
    }

    create() {
        this.makeView();
        this.initialize();
    }

    makeView() {
        let backgroundFar = this.game.add.tileSprite(0, 0, this.game.width, this.game.height, 'background_far');
        backgroundFar.autoScroll(-3, -1);
        //let backgroundNear = this.game.add.tileSprite(0, 0, this.game.width, this.game.height, 'background_near');
        //backgroundNear.autoScroll(-15, 0);

        this.game.scale.scaleMode = Phaser.ScaleManager.SHOW_ALL;
        this.game.scale.refresh();

        window.addEventListener('resize', () => {
            this.game.scale.refresh();
        });

        this.main.objects.ping_info_text =
            this.game.add.text(760, 10, 'Ping: ', {fill: '#FFFFFF', fontSize: 10});

        const refreshButton = this.game.add.button(20, 20, 'refresh');
        refreshButton.anchor.set(0.5);
        refreshButton.width = refreshButton.height = 30;
        refreshButton.onInputUp.add(() => {
            this.main.disconnect();
            this.game.state.start('boot');
        }, this);
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

        this.main.ee.on('Ping', (data) => {
            this.main.objects.ping_info_text.text = `Ping: ${data}\nFPS: ${this.game.time.fps}`;
        });

        const name = await this.enterName();

        this.main.send({
            RequestInitial: name
        });

        const initial = await this.main.waitForEvent('Initial');
        this.main.data.id = initial;

        this.main.send('Join');
        this.game.state.start('room', false, false, name);
    }
}
