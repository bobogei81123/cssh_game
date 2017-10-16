import Main from '../main';
import * as Phaser from 'phaser-ce';
import Bullet, {Hit} from '../objects/bullet';
import {GAME} from '../constant';
import User from '../objects/user';

const Point = Phaser.Point;
type Point = Phaser.Point;

enum Substate {
    Initialize,
    Ready,
    Fire,
    Resolving,
}

export class Start extends Phaser.State {
    substate: Substate;

    constructor(public main: Main) {
        super();
        this.substate = Substate.Initialize;
    }

    init() {
        this.registEvents();
    }

    create() {
        this.initialize();
    }

    async initialize() {
        this.main.send('RequestPlayersData');
        const data = await this.main.waitForEvent('PlayersData');
        this.main.data.syncWith(data);
        this.main.ee.on('PlayersData', this.main.data.syncWith);
        setTimeout(() => {
            this.substate = Substate.Ready;
        }, 2000);
    }

    registEvents() {
        this.main.ee.on('Fire', (data) => {
            console.log(data);
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
    }

    setProblemHTML(question, answers) {
        const $modal = document.getElementById('problem-modal');
        const $question = document.getElementById('question');
        const $answers = document.getElementById('answers');

        $question.innerText = question;
        while ($answers.firstChild) $answers.removeChild($answers.firstChild);

        answers.forEach((answer, idx) => {
            const $article = document.createElement("article");
            $article.classList.add('message');
            $article.classList.add('is-primary');

            const $content = document.createElement("div");
            $content.classList.add('message-body');
            $content.innerText = `(${idx+1}) ${answer}`;

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

    startFire() {
        console.log('start fire');
        return new Promise((resolve, reject) => {
            this.input.onDown.addOnce(() => {
                const promise = new Promise((resolve, reject) => {
                    const me = this.main.data.me();
                    me.startSpin();
                    this.input.onDown.addOnce(() => resolve(me));
                }).then((me: User) => {
                    const [pos, angle] = me.stopSpin();
                    this.main.send({
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

    update() {
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
                this.substate = Substate.Resolving;
                (async () => {
                    await this.startFire();
                    setTimeout(() => { this.substate = Substate.Ready; }, 1000);
                })();
            }
            case Substate.Resolving:
            case Substate.Initialize:
                break;
        }
    }
}
