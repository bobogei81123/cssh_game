import Main from '../main';
import * as Phaser from 'phaser-ce';
import RoomDataShower from '../objects/room_data_shower';
import PrepareButton from '../objects/prepare_button';

export class Room extends Phaser.State {
    data_shower: RoomDataShower;
    button: PrepareButton;
    func: Function;

    constructor(public main: Main) {
        super();
    }

    init(name: string) {
        this.data_shower = new RoomDataShower(this.game);
        this.button = new PrepareButton(this.game, 400, 520);
        this.button.onInputUp.addOnce(() => {
            this.main.send('Ready');
        });
        this.game.world.addChild(this.data_shower);
        this.game.world.addChild(this.button);

        this.func = (data) => {
            this.data_shower.updateWithData(data);
        }
        this.main.ee.on('RoomData', this.func);

        (async () => {
            this.main.send({
                'Join': name,
            });
            await this.main.waitForEvent('GameStart');
            this.game.state.start('start', false);
        })();
    }

    shutdown() {
        this.main.ee.off('RoomData', this.func);
        this.game.world.removeChild(this.data_shower);
        this.game.world.removeChild(this.button);
    }
}
