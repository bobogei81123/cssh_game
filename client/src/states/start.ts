import Main from '../main';
import * as Phaser from 'phaser-ce';
import Bullet, {Hit} from '../objects/bullet';
import BackButton from '../objects/back_button';
import {GAME} from '../constant';
import User from '../objects/user';
import * as marked from 'marked';
import * as _ from 'lodash';
import {PlayersData} from '../server_data/start';
import Channel from 'async-csp';

const renderer = new marked.Renderer();
renderer.image = function (href, title, alt) {
    var exec = /(.*) =(\d*%?)x(\d*%?)( \w*)?$/.exec(href);
    console.log(href, exec);
    if (exec) {
        var res = '<img src="' + exec[1] + '" alt="' + alt;
        if (exec[2]) res += '" height="' + exec[2];
        if (exec[3]) res += '" width="' + exec[3];
        if (exec[4] == " center")
            res += '" style="display: block; margin: 0 auto'
        return res + '">';
    }
    return `<img src="${href}" alt="${alt}">`;
}

const Point = Phaser.Point;
type Point = Phaser.Point;

enum Substate {
    Initialize,
    Ready,
    Fire,
    Resolving,
    End,
}

const EVENTS = [
    'PlayersData',
    'Problem',
    'StartFire',
    'FireResult',
    'Damage',
    'PlayersData',
    'TeamWin',
    'Dead',
    'JudgeResult',
]

export class Start extends Phaser.State {
    substate: Substate;
    dead: boolean;
    eventQueue: Channel;
    ended: boolean;

    constructor(public main: Main) {
        super();
        this.substate = Substate.Initialize;
        this.dead = false;
        this.eventQueue = new Channel(100);
        this.ended = false;
    }

    init() {
        this.registEvents();
    }

    create() {
        this.initialize();
    }

    async initialize() {
        this.main.send('Entered');
        this.run_event_loop();
    }

    registEvents() {
        EVENTS.forEach((event) => {
            this.main.ee.on(event, (data) => {
                (async () => {
                    await this.eventQueue.put([event, data]);
                })();
            });
        });
    }

    async run_event_loop() {
        const funcs = {
            PlayersData: async (data: PlayersData) => {
                this.main.data.syncWith(data)
            },
            Problem: async (data) => {
                this.setProblemHTML(data);
                this.showProblem();
            },
            FireResult: async ({fire, target}) => {
                let bullet;
                if (target == null) {
                    bullet = new Bullet(this.game);
                } else {
                    bullet = new Bullet(this.game, new Hit(
                        this.main.data.players[target].position
                    ));
                }
                bullet.fire(new Point(fire.pos.x, fire.pos.y), fire.angle);
            },
            Damage: async ({target: id, value, health_after, dead_after}) => {
                this.main.data.setHealth(id, health_after, dead_after);
                if (dead_after && id == this.main.data.id) {
                    this.meDead();
                }
            },
            JudgeResult: async (result) => {
                this.showResult(result);
            },
            StartFire: async (result) => {
                this.startFire();
            },
            TeamWin: async (team) => {
                this.teamWin(team);
            },
        };

        while (!this.ended) {
            const obj = await this.eventQueue.take();
            if (obj === Channel.DONE) {
                console.warn("Event queue stopped...");
                this.game.state.start('boot');
            }

            const [event, ...params] = obj;
            if (!(event in funcs)) {
                console.warn(`event ${event} not found`);
                continue;
            }

            await funcs[event](...params);
        }
        //this.main.ee.on('PlayersData'
        //this.main.ee.on('PlayersData', (data: PlayersData) => this.main.data.syncWith(data));

        //this.main.ee.on('Problem', (data) => {
            //this.setProblemHTML(data);
            //this.showProblem();
        //});
        
        //this.main.ee.on('FireResult', (data) => {
            //const {fire, damage} = data;
            //let bullet;
            //if (damage == null) {
                //bullet = new Bullet(this.game);
            //} else {
                //bullet = new Bullet(this.game, new Hit(
                    //this.main.data.players[damage.target].position, () => {
                    //this.main.data.setHealth(damage.target, damage.health_after);
                //}));
            //}
            //bullet.fire(new Point(fire.pos.x, fire.pos.y), fire.angle);
        //});

        //this.main.ee.on('TeamWin', (team) => {
            //const sprite = this.game.add.sprite(400, 300, (team == this.main.data.team ? 'win' : 'lose'));
            //sprite.scale.set(1.5);
            //sprite.anchor.set(0.5);
            //this.hideProblem();
            //const button = new BackButton(this.game, 400, 540);
            //button.onInputUp.addOnce(() => {
                //sprite.destroy();
                //this.game.world.removeChild(button);
                //this.game.state.start('boot');
            //});
            //this.game.world.addChild(button);
        //});

        //this.main.ee.on('Dead', (id) => {
            //let p = this.main.data.players[id];
            //if (p != null) {
                //p.markDead();
            //}

            //if (id == this.main.data.id) {
                //this.dead = true;
                //this.substate = Substate.End;
                //this.hideProblem();
            //}
        //});

        //this.main.ee.on('JudgeResult', (result) => {
            //this.showResult(result);
        //});

        //this.main.ee.on('StartFire', (result) => {
            //this.startFire();
        //});
    }

    teamWin(team: number) {
        this.hideProblem();
        const sprite = this.game.add.sprite(400, 300, (team == this.main.data.team ? 'win' : 'lose'));
        sprite.scale.set(1.5);
        sprite.anchor.set(0.5);
        this.ended = true;
    }

    meDead() {
        this.hideProblem();
        const hint = this.game.add.text(400, 40, '你的戰機已被摧毀', {fill: 'white', fontSize: 20});
        hint.anchor.set(0.5);
    }

    setProblemHTML({question, answers}) {
        const $modal = document.getElementById('problem-modal');
        const $question = document.getElementById('question');
        const $answers = document.getElementById('answers');

        $question.innerHTML = marked(question, {renderer});
        while ($answers.firstChild) $answers.removeChild($answers.firstChild);

        answers.forEach((answer, idx) => {
            const $article = document.createElement("article");
            $article.classList.add('message');
            $article.classList.add('is-primary');

            const $content = document.createElement("div");
            $content.classList.add('message-body');

            answer = `(${idx+1}) ` + answer;
            const mdText = marked(answer, {renderer});
            $content.insertAdjacentHTML('beforeend', mdText);

            $article.appendChild($content);
            $article.onclick = (() => {
                this.answer(idx);
            });
            $answers.appendChild($article);
        });
    }

    showProblem() {
        const $modal = document.getElementById('problem-modal');
        $modal.classList.add('is-active');
    }

    hideProblem() {
        const $modal = document.getElementById('problem-modal');
        $modal.classList.remove('is-active');
    }

    answer(idx: number) {
        this.hideProblem();
        this.main.send({
            Answer: idx,
        });
    }

    showResult(flag) {
        const sprite = this.game.add.sprite(GAME.WIDTH/2, GAME.HEIGHT/2, flag ? 'check' : 'cross');
        sprite.anchor.set(0.5);
        const tween = this.game.add.tween(sprite).to({alpha: 0}, 1200, Phaser.Easing.Quadratic.In); 
        tween.onComplete.add(() => {
            sprite.destroy();
        });
        tween.start();
    }

    async startFire() {
        console.log('start fire');
        const hint = this.game.add.text(400, 20, '點擊任意一處開始描準', {fill: 'white', fontSize: 20});
        hint.anchor.set(0.5);

        const generateClickPromise = () => (new Promise((resolve, reject) => {
            this.input.onDown.addOnce(() => {
                resolve();
            });
        }));

        await generateClickPromise();

        hint.text = '再次點擊往箭頭方向射擊';
        
        const me = this.main.data.me();
        const [pos, angle] = await me.startSpin(generateClickPromise());

        hint.text = '';

        await me.rotateAndFire(angle);
        this.main.send({
            Fire: {
                pos: pos,
                angle: angle,
            }
        });
    }

    update() {
        /*
        switch (this.substate) {
            case Substate.Ready: {
                this.substate = Substate.Resolving;
                (async () => {
                    this.main.send('RequestProblem');
                    const {question, answers} = await this.main.waitForEvent('Problem');
                    this.setProblemHTML(question, answers);
                    this.showProblem();

                    const my_answer = await this.main.waitForEvent('chooseAnswer');
                    this.hideProblem();
                    this.main.send({'Answer': my_answer});

                    const result = await this.main.waitForEvent('JudgeResult');
                    if (result) {
                        this.showResult(true);
                        this.substate = Substate.Fire;
                    } else {
                        this.showResult(false);
                        setTimeout(() => {this.substate = Substate.Ready;}, 3000);
                    }
                })();
                break;
            }
            case Substate.Fire: {
                if (this.dead) {
                    this.substate = Substate.End;
                    return;
                }
                this.substate = Substate.Resolving;
                (async () => {
                    await this.startFire();
                    setTimeout(() => { this.substate = Substate.Ready; }, 1000);
                })();
            }
            case Substate.Resolving:
            case Substate.Initialize:
            case Substate.End:
                break;
        }
         */
    }

    shutdown() {
        this.main.ee.removeAllListeners('PlayersData');
    }
}
