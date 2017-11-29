import Main from '../main';
import * as Phaser from 'phaser-ce';
import Bullet, {Hit} from '../objects/bullet';
import BackButton from '../objects/back_button.ts';
import {GAME} from '../constant';
import User from '../objects/user';
import * as marked from 'marked';
import * as _ from 'lodash';

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

export class Start extends Phaser.State {
    substate: Substate;
    dead: boolean;

    constructor(public main: Main) {
        super();
        this.substate = Substate.Initialize;
        this.dead = false;
    }

    init() {
        this.registEvents();
    }

    create() {
        this.initialize();
    }

    async initialize() {
        /*
        this.main.send('RequestPlayersData');
        const data = await this.main.waitForEvent('PlayersData');
        if (_.includes(data.teams[0], this.main.data.id)) {
            this.main.data.team = 0;
        } else {
            this.main.data.team = 1;
        }
        this.main.data.syncWith(data);
        this.main.ee.on('PlayersData', (data) => this.main.data.syncWith(data));
        setTimeout(() => {
            this.substate = Substate.Ready;
        }, 2000);
         */
    }

    registEvents() {
        /*
        this.main.ee.on('Fire', (data) => {
            const {fire, damage} = data;
            let bullet;
            if (damage == null) {
                bullet = new Bullet(this.game);
            } else {
                bullet = new Bullet(this.game, new Hit(
                    this.main.data.players[damage.target].position, () => {
                    this.main.data.setHealth(damage.target, damage.health_after);
                }));
            }
            bullet.fire(new Point(fire.pos.x, fire.pos.y), fire.angle);
        });

        this.main.ee.on('TeamWin', (team) => {
            const sprite = this.game.add.sprite(400, 300, (team == this.main.data.team ? 'win' : 'lose'));
            sprite.scale.set(1.5);
            sprite.anchor.set(0.5);
            this.hideProblem();
            const button = new BackButton(this.game, 400, 540);
            button.onInputUp.addOnce(() => {
                sprite.destroy();
                this.game.world.removeChild(button);
                this.game.state.start('boot');
            });
            this.game.world.addChild(button);
        });

        this.main.ee.on('Dead', (id) => {
            let p = this.main.data.players[id];
            if (p != null) {
                p.markDead();
            }

            if (id == this.main.data.id) {
                this.dead = true;
                this.substate = Substate.End;
                this.hideProblem();
            }
        });
         */
    }

    setProblemHTML(question, answers) {
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
                this.main.ee.emit('chooseAnswer', idx);
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
