import Main from '../main';
import * as Phaser from 'phaser-ce';
import RoomDataShower from '../objects/room_data_shower';
import PrepareButton from '../objects/prepare_button';

export interface Player {
    id: number;
    name: string;
    team: number;
    ready: boolean;
}

export interface RoomData {
    players: {
        [_: number]: Player;
    };
    teams: [number[], number[]];
}

export class Room extends Phaser.State {
    data_shower: RoomDataShower;
    button: PrepareButton;
    func: Function;

    constructor(public main: Main) {
        super();
    }

    init(name: string) {
        this.main.send('Entered');

        this.data_shower = new RoomDataShower(this.game);
        this.button = new PrepareButton(this.game, 400, 520);

        this.button.onInputUp.addOnce(() => {
            this.main.send('Ready');
        });
        this.game.world.addChild(this.data_shower);
        this.game.world.addChild(this.button);

        this.func = (data: RoomData) => {
            const id = this.main.data.id;
            if (!(id in data.players)) {
                return;
            }

            if (data.players[id].team == 1) {
                [data.teams[0], data.teams[1]] = 
                    [data.teams[1], data.teams[0]]
            }
            this.data_shower.updateWithData(data as any);
        }
        this.main.ee.on('RoomData', this.func);

        (async () => {
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
